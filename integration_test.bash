#!/usr/bin/env bash

# http://redsymbol.net/articles/unofficial-bash-strict-mode/
set -euo pipefail
IFS=$'\n\t'

finish() {
  exit_code=$?

  echo
  if [[ $exit_code -eq 0 ]]; then
    echo "INTEGRATION TESTS SUCCEDED"
  else
    echo "INTEGRATION TESTS FAILED"
  fi
}
trap finish EXIT

CONTRACT_PATH="artifacts/lido_satellite.wasm"
MAIN_WALLET="demowallet1"
MAIN_WALLET_ADDR="neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2"
SECOND_WALLET="demowallet2"
SECOND_WALLET_ADDR="neutron10h9stc5v6ntgeygf5xf945njqq5h32r54rf7kf"
NEUTRON_HOME="../neutron/data/test-1"
NEUTRON_NODE="tcp://0.0.0.0:26657"
CHAIN_ID="test-1"

declare -a tx=(
  "--home" "$NEUTRON_HOME"
  "--keyring-backend" "test"
  "--broadcast-mode" "block"
  "--gas" "auto"
  "--gas-adjustment" "1.3"
  "--gas-prices" "0.0025untrn"
  "--node" "$NEUTRON_NODE"
  "--chain-id" "$CHAIN_ID"
  "--output" "json"
  "-y"
)

declare -a q=(
  "--node" "$NEUTRON_NODE"
  "--output" "json"
)

select_attr() {
  printf '.logs[0].events[] | select(.type == "%s").attributes[] | select(.key == "%s").value' "$1" "$2"
}

assert_success() {
  local code
  code="$(jq -r '.code' </dev/stdin)"
  if [[ $code -ne 0 ]]; then
    echo "tx failed"
    exit 1
  fi
}

get_balance() {
  neutrond query bank balances "$1" --denom "$2" "${q[@]}" | jq -r '.amount'
}

assert_balance() {
  local amount
  amount="$(get_balance "$1" "$2")"
  if [[ $amount -ne $3 ]]; then
      echo "incorrect amount"
      exit 1
  fi
  echo "$1 has correct amount of $2: $amount"
}

code_id="$(neutrond tx wasm store "$CONTRACT_PATH" --from "$MAIN_WALLET" "${tx[@]}" | jq -r "$(select_attr "store_code" "code_id")")"
echo "Code ID: $code_id"

msg='{"wsteth_denom":"uibcatom","subdenom":"steth"}'
contract_address="$(neutrond tx wasm instantiate "$code_id" "$msg" --amount 1000000untrn --no-admin --label steth --from "$MAIN_WALLET" "${tx[@]}" | jq -r "$(select_attr "instantiate" "_contract_address")")"
echo "Contract address: $contract_address"

main_balance_before="$(get_balance "$MAIN_WALLET_ADDR" "uibcatom")"
second_balance_before="$(get_balance "$SECOND_WALLET_ADDR" "uibcatom")"
echo "Balance of main wallet: $main_balance_before"
echo "Balance of second wallet: $second_balance_before"

echo
echo "Mint 1000 steth to main account"
neutrond tx wasm execute "$contract_address" '{"mint":{}}' --amount 1000uibcatom --from "$MAIN_WALLET" "${tx[@]}" | assert_success
assert_balance "$MAIN_WALLET_ADDR" "factory/$contract_address/steth" "1000"
assert_balance "$contract_address" "uibcatom" "1000"

echo
echo "Mint 2000 more steth to main account"
neutrond tx wasm execute "$contract_address" '{"mint":{}}' --amount 2000uibcatom --from "$MAIN_WALLET" "${tx[@]}" | assert_success
assert_balance "$MAIN_WALLET_ADDR" "factory/$contract_address/steth" "3000"
assert_balance "$contract_address" "uibcatom" "3000"

echo
echo "Mint 1500 steth to second account"
neutrond tx wasm execute "$contract_address" '{"mint":{}}' --amount 1500uibcatom --from "$SECOND_WALLET" "${tx[@]}" | assert_success
assert_balance "$MAIN_WALLET_ADDR" "factory/$contract_address/steth" "3000"
assert_balance "$SECOND_WALLET_ADDR" "factory/$contract_address/steth" "1500"
assert_balance "$contract_address" "uibcatom" "4500"

echo
echo "Burn 200 steth from main account and send wsteth to second account"
msg="$(printf '{"burn":{"receiver":"%s"}}' "$SECOND_WALLET_ADDR")"
neutrond tx wasm execute "$contract_address" "$msg" --amount "200factory/$contract_address/steth" --from "$MAIN_WALLET" "${tx[@]}" | assert_success
assert_balance "$MAIN_WALLET_ADDR" "factory/$contract_address/steth" "2800"
assert_balance "$SECOND_WALLET_ADDR" "factory/$contract_address/steth" "1500"
assert_balance "$contract_address" "uibcatom" "4300"

echo
echo "Send 300 steth from main account to second account"
neutrond tx bank send "$MAIN_WALLET" "$SECOND_WALLET_ADDR" "300factory/$contract_address/steth" "${tx[@]}" | assert_success
assert_balance "$MAIN_WALLET_ADDR" "factory/$contract_address/steth" "2500"
assert_balance "$SECOND_WALLET_ADDR" "factory/$contract_address/steth" "1800"
assert_balance "$contract_address" "uibcatom" "4300"

echo
echo "Burn 1800 steth from second account"
neutrond tx wasm execute "$contract_address" '{"burn":{}}' --amount "1800factory/$contract_address/steth" --from "$SECOND_WALLET" "${tx[@]}" | assert_success
assert_balance "$MAIN_WALLET_ADDR" "factory/$contract_address/steth" "2500"
assert_balance "$SECOND_WALLET_ADDR" "factory/$contract_address/steth" "0"
assert_balance "$contract_address" "uibcatom" "2500"

echo
main_balance_after="$(get_balance "$MAIN_WALLET_ADDR" "uibcatom")"
second_balance_after="$(get_balance "$SECOND_WALLET_ADDR" "uibcatom")"
echo "Balance of main wallet: $main_balance_after"
echo "Balance of second wallet: $second_balance_after"

# I really don't want to trust bash my precious long arithmetics
main_diff="$(python -c "print(${main_balance_after}-${main_balance_before})")"
second_diff="$(python -c "print(${second_balance_after}-${second_balance_before})")"

echo
if [[ $main_diff -eq -3000 ]]; then
  echo "Main wallet has lost 3000 wsteth"
else
  echo "Main wallet wsteth diff: $main_diff, expected diff: -3000"
  exit 1
fi
if [[ $second_diff -eq 500 ]]; then
  echo "Second wallet has earned 500 wsteth"
else
  echo "Second wallet wsteth diff: $second_diff, expected diff: 500"
  exit 1
fi
