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
MAIN_WALLET_ADDR_NEUTRON="neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2"
MAIN_WALLET_ADDR_GAIA="cosmos1m9l358xunhhwds0568za49mzhvuxx9uxre5tud"
SECOND_WALLET="demowallet2"
SECOND_WALLET_ADDR_NEUTRON="neutron10h9stc5v6ntgeygf5xf945njqq5h32r54rf7kf"
NEUTRON_HOME="../neutron/data/test-1"
NEUTRON_NODE="tcp://0.0.0.0:26657"
NEUTRON_CHAIN_ID="test-1"
GAIA_HOME="../neutron/data/test-2"
GAIA_NODE="tcp://0.0.0.0:16657"
GAIA_CHAIN_ID="test-2"
ATOM_ON_NEUTRON_IBC_DENOM="ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2"

declare -a ntx=(
  "--home" "$NEUTRON_HOME"
  "--keyring-backend" "test"
  "--broadcast-mode" "sync"
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
  "--broadcast-mode" "sync"
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

declare -a gq=(
  "--node" "$GAIA_NODE"
  "--output" "json"
)

wait_gtx() {
  wait_tx "gaiad" "gq"
}

wait_ntx() {
  wait_tx "neutrond" "nq"
}

wait_tx() {
  local aname
  local q
  local txhash
  local attempts
  aname="$2[@]"
  q=("${!aname}")
  txhash="$(jq -r '.txhash' </dev/stdin)"
  ((attempts=50))
  while ! "$1" query tx --type=hash "$txhash" "${q[@]}" 2>/dev/null; do
    ((attempts-=1)) || {
      echo "tx $txhash still not included in block" 1>&2
      exit 1
    }
    sleep 0.1
  done
}

select_attr() {
  printf '.logs[0].events[] | select(.type == "%s").attributes[] | select(.key == "%s").value' "$1" "$2"
}

assert_success() {
  local tx_status
  local code
  tx_status="$(cat /dev/stdin)"
  code="$(echo "$tx_status" | jq -r '.code')"
  if [[ $code -ne 0 ]]; then
    echo "[FAIL] tx failed:"
    echo "$tx_status" | jq
    exit 1
  fi
}

get_balance_neutron() {
  neutrond query bank balances "$1" --denom "$2" "${nq[@]}" | jq -r '.amount'
}

assert_balance() {
  if [[ $3 -ne $4 ]]; then
    echo "[FAIL] $1 has incorrect amount of $2, got: $3, expected: $4"
    exit 1
  fi
  echo "[OK] $1 has correct amount of $2: $3"
}

assert_balance_neutron() {
  local amount
  amount="$(get_balance_neutron "$1" "$2")"
  assert_balance "$1" "$2" "$amount" "$3"
}

echo -n "Prepare IBC funds: "
seq="$(gaiad query account "$MAIN_WALLET_ADDR_GAIA" "${gq[@]}" | jq -r '.sequence')"
if [[ "$(get_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "$ATOM_ON_NEUTRON_IBC_DENOM")" -lt 20000 ]]; then
  gaiad tx ibc-transfer transfer "transfer" "channel-0" "$MAIN_WALLET_ADDR_NEUTRON" 20000uatom --from demowallet1 "${gtx[@]}" -s "$seq" >/dev/null
  ((seq+=1))
fi
if [[ "$(get_balance_neutron "$SECOND_WALLET_ADDR_NEUTRON" "$ATOM_ON_NEUTRON_IBC_DENOM")" -lt 10000 ]]; then
  gaiad tx ibc-transfer transfer "transfer" "channel-0" "$SECOND_WALLET_ADDR_NEUTRON" 10000uatom --from demowallet1 "${gtx[@]}" -s "$seq" >/dev/null
fi
((attempts=200))
while [[ "$(get_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "$ATOM_ON_NEUTRON_IBC_DENOM")" -lt 20000 ]]; do
  echo -n "."
  ((attempts=attempts-1)) || {
    echo "seems like IBC transfer failed" 1>&2
    exit 1
  }
  sleep 0.1
done
while [[ "$(get_balance_neutron "$SECOND_WALLET_ADDR_NEUTRON" "$ATOM_ON_NEUTRON_IBC_DENOM")" -lt 10000 ]]; do
  echo -n "."
  ((attempts=attempts-1)) || {
    echo "seems like IBC transfer failed" 1>&2
    exit 1
  }
  sleep 0.1
done
echo " done"

main_balance_before="$(get_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "$ATOM_ON_NEUTRON_IBC_DENOM")"
second_balance_before="$(get_balance_neutron "$SECOND_WALLET_ADDR_NEUTRON" "$ATOM_ON_NEUTRON_IBC_DENOM")"
echo "Balance of main wallet: $main_balance_before"
echo "Balance of second wallet: $second_balance_before"
echo

code_id="$(neutrond tx wasm store "$CONTRACT_PATH" --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | jq -r "$(select_attr "store_code" "code_id")")"
echo "Code ID: $code_id"

msg="$(printf '{"bridged_denom":"%s","canonical_subdenom":"steth"}' "$ATOM_ON_NEUTRON_IBC_DENOM")"
contract_address="$(neutrond tx wasm instantiate "$code_id" "$msg" --amount 1000000untrn --no-admin --label steth --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | jq -r "$(select_attr "instantiate" "_contract_address")")"
echo "Contract address: $contract_address"

echo
echo "Mint 1000 steth to main account"
neutrond tx wasm execute "$contract_address" '{"mint":{}}' --amount "1000$ATOM_ON_NEUTRON_IBC_DENOM" --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | assert_success
assert_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "factory/$contract_address/steth" "1000"
assert_balance_neutron "$contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "1000"

echo
echo "Mint 2000 more steth to main account"
neutrond tx wasm execute "$contract_address" '{"mint":{}}' --amount "2000$ATOM_ON_NEUTRON_IBC_DENOM" --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | assert_success
assert_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "factory/$contract_address/steth" "3000"
assert_balance_neutron "$contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "3000"

echo
echo "Mint 1500 steth to second account"
neutrond tx wasm execute "$contract_address" '{"mint":{}}' --amount "1500$ATOM_ON_NEUTRON_IBC_DENOM" --from "$SECOND_WALLET" "${ntx[@]}" | wait_ntx | assert_success
assert_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "factory/$contract_address/steth" "3000"
assert_balance_neutron "$SECOND_WALLET_ADDR_NEUTRON" "factory/$contract_address/steth" "1500"
assert_balance_neutron "$contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "4500"

echo
echo "Burn 200 steth from main account and send $ATOM_ON_NEUTRON_IBC_DENOM to second account"
msg="$(printf '{"burn":{"receiver":"%s"}}' "$SECOND_WALLET_ADDR_NEUTRON")"
neutrond tx wasm execute "$contract_address" "$msg" --amount "200factory/$contract_address/steth" --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | assert_success
assert_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "factory/$contract_address/steth" "2800"
assert_balance_neutron "$SECOND_WALLET_ADDR_NEUTRON" "factory/$contract_address/steth" "1500"
assert_balance_neutron "$contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "4300"

echo
echo "Send 300 steth from main account to second account"
neutrond tx bank send "$MAIN_WALLET" "$SECOND_WALLET_ADDR_NEUTRON" "300factory/$contract_address/steth" "${ntx[@]}" | wait_ntx | assert_success
assert_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "factory/$contract_address/steth" "2500"
assert_balance_neutron "$SECOND_WALLET_ADDR_NEUTRON" "factory/$contract_address/steth" "1800"
assert_balance_neutron "$contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "4300"

echo
echo "Burn 1800 steth from second account"
neutrond tx wasm execute "$contract_address" '{"burn":{}}' --amount "1800factory/$contract_address/steth" --from "$SECOND_WALLET" "${ntx[@]}" | wait_ntx | assert_success
assert_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "factory/$contract_address/steth" "2500"
assert_balance_neutron "$SECOND_WALLET_ADDR_NEUTRON" "factory/$contract_address/steth" "0"
assert_balance_neutron "$contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "2500"

echo
main_balance_after="$(get_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "$ATOM_ON_NEUTRON_IBC_DENOM")"
second_balance_after="$(get_balance_neutron "$SECOND_WALLET_ADDR_NEUTRON" "$ATOM_ON_NEUTRON_IBC_DENOM")"
echo "Balance of main wallet: $main_balance_after"
echo "Balance of second wallet: $second_balance_after"

# I really don't want to trust bash my precious long arithmetics
main_diff="$(python -c "print(${main_balance_after}-${main_balance_before})")"
second_diff="$(python -c "print(${second_balance_after}-${second_balance_before})")"

echo
if [[ $main_diff -eq -3000 ]]; then
  echo "[OK] Main wallet has lost 3000 $ATOM_ON_NEUTRON_IBC_DENOM"
else
  echo "[FAIL] Main wallet $ATOM_ON_NEUTRON_IBC_DENOM diff: $main_diff, expected diff: -3000"
  exit 1
fi
if [[ $second_diff -eq 500 ]]; then
  echo "[OK] Second wallet has earned 500 $ATOM_ON_NEUTRON_IBC_DENOM"
else
  echo "[FAIL] Second wallet $ATOM_ON_NEUTRON_IBC_DENOM diff: $second_diff, expected diff: 500"
  exit 1
fi
