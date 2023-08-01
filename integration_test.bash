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
NEUTRON_CHAIN_ID="test-1"
GAIA_HOME="../neutron/data/test-2"
GAIA_NODE="tcp://0.0.0.0:16657"
GAIA_CHAIN_ID="test-2"
IBC_DENOM="ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2"

declare -a ntx=(
  "--home" "$NEUTRON_HOME"
  "--keyring-backend" "test"
  "--broadcast-mode" "block"
  "--gas" "10000000"
  "--gas-prices" "0.0025untrn"
  "--node" "$NEUTRON_NODE"
  "--chain-id" "$NEUTRON_CHAIN_ID"
  "--output" "json"
  "-y"
)

declare -a gtx=(
  "--home" "$GAIA_HOME"
  "--keyring-backend" "test"
  "--broadcast-mode" "block"
  "--gas" "10000000"
  "--gas-prices" "0.0025uatom"
  "--node" "$GAIA_NODE"
  "--chain-id" "$GAIA_CHAIN_ID"
  "--output" "json"
  "-y"
)

declare -a nq=(
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
    echo "[FAIL] tx failed"
    exit 1
  fi
}

get_balance() {
  neutrond query bank balances "$1" --denom "$2" "${nq[@]}" | jq -r '.amount'
}

assert_balance() {
  local amount
  amount="$(get_balance "$1" "$2")"
  if [[ $amount -ne $3 ]]; then
      echo "[FAIL] incorrect amount"
      exit 1
  fi
  echo "[OK] $1 has correct amount of $2: $amount"
}

echo -n "Prepare IBC funds…"
gaiad tx ibc-transfer transfer "transfer" "channel-0" "$MAIN_WALLET_ADDR" 20000uatom --from demowallet1 "${gtx[@]}" | assert_success
gaiad tx ibc-transfer transfer "transfer" "channel-0" "$SECOND_WALLET_ADDR" 10000uatom --from demowallet1 "${gtx[@]}" | assert_success
echo " done"

echo -n "Waiting 10 seconds for IBC transfers to complete"
# shellcheck disable=SC2034
for i in $(seq 10); do
  sleep 1
  echo -n .
done
echo " done"

main_balance_before="$(get_balance "$MAIN_WALLET_ADDR" "$IBC_DENOM")"
second_balance_before="$(get_balance "$SECOND_WALLET_ADDR" "$IBC_DENOM")"
echo "Balance of main wallet: $main_balance_before"
echo "Balance of second wallet: $second_balance_before"
echo

code_id="$(neutrond tx wasm store "$CONTRACT_PATH" --from "$MAIN_WALLET" "${ntx[@]}" | jq -r "$(select_attr "store_code" "code_id")")"
echo "Code ID: $code_id"

msg="$(printf '{"bridged_denom":"%s","canonical_subdenom":"steth"}' "$IBC_DENOM")"
contract_address="$(neutrond tx wasm instantiate "$code_id" "$msg" --amount 1000000untrn --no-admin --label steth --from "$MAIN_WALLET" "${ntx[@]}" | jq -r "$(select_attr "instantiate" "_contract_address")")"
echo "Contract address: $contract_address"

echo
echo "Mint 1000 steth to main account"
neutrond tx wasm execute "$contract_address" '{"mint":{}}' --amount "1000$IBC_DENOM" --from "$MAIN_WALLET" "${ntx[@]}" | assert_success
assert_balance "$MAIN_WALLET_ADDR" "factory/$contract_address/steth" "1000"
assert_balance "$contract_address" "$IBC_DENOM" "1000"

echo
echo "Mint 2000 more steth to main account"
neutrond tx wasm execute "$contract_address" '{"mint":{}}' --amount "2000$IBC_DENOM" --from "$MAIN_WALLET" "${ntx[@]}" | assert_success
assert_balance "$MAIN_WALLET_ADDR" "factory/$contract_address/steth" "3000"
assert_balance "$contract_address" "$IBC_DENOM" "3000"

echo
echo "Mint 1500 steth to second account"
neutrond tx wasm execute "$contract_address" '{"mint":{}}' --amount "1500$IBC_DENOM" --from "$SECOND_WALLET" "${ntx[@]}" | assert_success
assert_balance "$MAIN_WALLET_ADDR" "factory/$contract_address/steth" "3000"
assert_balance "$SECOND_WALLET_ADDR" "factory/$contract_address/steth" "1500"
assert_balance "$contract_address" "$IBC_DENOM" "4500"

echo
echo "Burn 200 steth from main account and send $IBC_DENOM to second account"
msg="$(printf '{"burn":{"receiver":"%s"}}' "$SECOND_WALLET_ADDR")"
neutrond tx wasm execute "$contract_address" "$msg" --amount "200factory/$contract_address/steth" --from "$MAIN_WALLET" "${ntx[@]}" | assert_success
assert_balance "$MAIN_WALLET_ADDR" "factory/$contract_address/steth" "2800"
assert_balance "$SECOND_WALLET_ADDR" "factory/$contract_address/steth" "1500"
assert_balance "$contract_address" "$IBC_DENOM" "4300"

echo
echo "Send 300 steth from main account to second account"
neutrond tx bank send "$MAIN_WALLET" "$SECOND_WALLET_ADDR" "300factory/$contract_address/steth" "${ntx[@]}" | assert_success
assert_balance "$MAIN_WALLET_ADDR" "factory/$contract_address/steth" "2500"
assert_balance "$SECOND_WALLET_ADDR" "factory/$contract_address/steth" "1800"
assert_balance "$contract_address" "$IBC_DENOM" "4300"

echo
echo "Burn 1800 steth from second account"
neutrond tx wasm execute "$contract_address" '{"burn":{}}' --amount "1800factory/$contract_address/steth" --from "$SECOND_WALLET" "${ntx[@]}" | assert_success
assert_balance "$MAIN_WALLET_ADDR" "factory/$contract_address/steth" "2500"
assert_balance "$SECOND_WALLET_ADDR" "factory/$contract_address/steth" "0"
assert_balance "$contract_address" "$IBC_DENOM" "2500"

echo
main_balance_after="$(get_balance "$MAIN_WALLET_ADDR" "$IBC_DENOM")"
second_balance_after="$(get_balance "$SECOND_WALLET_ADDR" "$IBC_DENOM")"
echo "Balance of main wallet: $main_balance_after"
echo "Balance of second wallet: $second_balance_after"

# I really don't want to trust bash my precious long arithmetics
main_diff="$(python -c "print(${main_balance_after}-${main_balance_before})")"
second_diff="$(python -c "print(${second_balance_after}-${second_balance_before})")"

echo
if [[ $main_diff -eq -3000 ]]; then
  echo "[OK] Main wallet has lost 3000 $IBC_DENOM"
else
  echo "[FAIL] Main wallet $IBC_DENOM diff: $main_diff, expected diff: -3000"
  exit 1
fi
if [[ $second_diff -eq 500 ]]; then
  echo "[OK] Second wallet has earned 500 $IBC_DENOM"
else
  echo "[FAIL] Second wallet $IBC_DENOM diff: $second_diff, expected diff: 500"
  exit 1
fi
