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

LIDO_SATELLITE_PATH="artifacts/lido_satellite.wasm"
WRAP_AND_SEND_PATH="artifacts/wrap_and_send.wasm"
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

declare -a gq=(
  "--node" "$GAIA_NODE"
  "--output" "json"
)

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

get_balance_gaia() {
  gaiad query bank balances "$1" --denom "$2" "${gq[@]}" | jq -r '.amount'
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

assert_balance_gaia() {
  local amount
  amount="$(get_balance_gaia "$1" "$2")"
  assert_balance "$1" "$2" "$amount" "$3"
}

echo -n "Prepare IBC funds…"
gaiad tx ibc-transfer transfer "transfer" "channel-0" "$MAIN_WALLET_ADDR_NEUTRON" 20000uatom --from demowallet1 "${gtx[@]}" | assert_success
gaiad tx ibc-transfer transfer "transfer" "channel-0" "$SECOND_WALLET_ADDR_NEUTRON" 10000uatom --from demowallet1 "${gtx[@]}" | assert_success
echo " done"

echo -n "Waiting 10 seconds for IBC transfers to complete"
# shellcheck disable=SC2034
for i in $(seq 10); do
  sleep 1
  echo -n .
done
echo " done"
main_balance_before="$(get_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "$ATOM_ON_NEUTRON_IBC_DENOM")"
second_balance_before="$(get_balance_neutron "$SECOND_WALLET_ADDR_NEUTRON" "$ATOM_ON_NEUTRON_IBC_DENOM")"

echo
lido_satellite_code_id="$(neutrond tx wasm store "$LIDO_SATELLITE_PATH" --from "$MAIN_WALLET" "${ntx[@]}" | jq -r "$(select_attr "store_code" "code_id")")"
echo "Lido Satellite Code ID: $lido_satellite_code_id"
msg="$(printf '{"bridged_denom":"%s","canonical_subdenom":"wATOM"}' "$ATOM_ON_NEUTRON_IBC_DENOM")"
lido_satellite_contract_address="$(neutrond tx wasm instantiate "$lido_satellite_code_id" "$msg" --amount 1000000untrn --no-admin --label lido_satellite --from "$MAIN_WALLET" "${ntx[@]}" | jq -r "$(select_attr "instantiate" "_contract_address")")"
echo "Lido Satellite Contract address: $lido_satellite_contract_address"

echo
echo "Mint 1000 wATOM to main account"
neutrond tx wasm execute "$lido_satellite_contract_address" '{"mint":{}}' --amount "1000$ATOM_ON_NEUTRON_IBC_DENOM" --from "$MAIN_WALLET" "${ntx[@]}" | assert_success
assert_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "factory/$lido_satellite_contract_address/wATOM" "1000"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "1000"

echo
echo "Mint 2000 more wATOM to main account"
neutrond tx wasm execute "$lido_satellite_contract_address" '{"mint":{}}' --amount "2000$ATOM_ON_NEUTRON_IBC_DENOM" --from "$MAIN_WALLET" "${ntx[@]}" | assert_success
assert_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "factory/$lido_satellite_contract_address/wATOM" "3000"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "3000"

echo
echo "Mint 1500 wATOM to second account"
neutrond tx wasm execute "$lido_satellite_contract_address" '{"mint":{}}' --amount "1500$ATOM_ON_NEUTRON_IBC_DENOM" --from "$SECOND_WALLET" "${ntx[@]}" | assert_success
assert_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "factory/$lido_satellite_contract_address/wATOM" "3000"
assert_balance_neutron "$SECOND_WALLET_ADDR_NEUTRON" "factory/$lido_satellite_contract_address/wATOM" "1500"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "4500"

echo
echo "Burn 200 wATOM from main account and send $ATOM_ON_NEUTRON_IBC_DENOM to second account"
msg="$(printf '{"burn":{"receiver":"%s"}}' "$SECOND_WALLET_ADDR_NEUTRON")"
neutrond tx wasm execute "$lido_satellite_contract_address" "$msg" --amount "200factory/$lido_satellite_contract_address/wATOM" --from "$MAIN_WALLET" "${ntx[@]}" | assert_success
assert_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "factory/$lido_satellite_contract_address/wATOM" "2800"
assert_balance_neutron "$SECOND_WALLET_ADDR_NEUTRON" "factory/$lido_satellite_contract_address/wATOM" "1500"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "4300"

echo
echo "Send 300 wATOM from main account to second account"
neutrond tx bank send "$MAIN_WALLET" "$SECOND_WALLET_ADDR_NEUTRON" "300factory/$lido_satellite_contract_address/wATOM" "${ntx[@]}" | assert_success
assert_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "factory/$lido_satellite_contract_address/wATOM" "2500"
assert_balance_neutron "$SECOND_WALLET_ADDR_NEUTRON" "factory/$lido_satellite_contract_address/wATOM" "1800"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "4300"

echo
echo "Burn 1800 wATOM from second account"
neutrond tx wasm execute "$lido_satellite_contract_address" '{"burn":{}}' --amount "1800factory/$lido_satellite_contract_address/wATOM" --from "$SECOND_WALLET" "${ntx[@]}" | assert_success
assert_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "factory/$lido_satellite_contract_address/wATOM" "2500"
assert_balance_neutron "$SECOND_WALLET_ADDR_NEUTRON" "factory/$lido_satellite_contract_address/wATOM" "0"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "2500"

main_balance_after="$(get_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "$ATOM_ON_NEUTRON_IBC_DENOM")"
second_balance_after="$(get_balance_neutron "$SECOND_WALLET_ADDR_NEUTRON" "$ATOM_ON_NEUTRON_IBC_DENOM")"
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

# Tests below will remain disabled until astroport router is mocked

#echo
#wrap_and_send_code_id="$(neutrond tx wasm store "$WRAP_AND_SEND_PATH" --from "$MAIN_WALLET" "${ntx[@]}" | jq -r "$(select_attr "store_code" "code_id")")"
#echo "Wrap and Send Code ID: $wrap_and_send_code_id"
#msg="$(printf '{"lido_satellite":"%s","ibc_fee_denom":"untrn"}' "$lido_satellite_contract_address")"
#wrap_and_send_contract_address="$(neutrond tx wasm instantiate "$wrap_and_send_code_id" "$msg" --amount 2000untrn --no-admin --label wrap_and_send --from "$MAIN_WALLET" "${ntx[@]}" | jq -r "$(select_attr "instantiate" "_contract_address")")"
#echo "Wrap and Send Contract address: $wrap_and_send_contract_address"
#
#echo
#echo "Mint 200 wATOM and send to Gaia"
#msg="$(printf '{"wrap_and_send":{"source_port":"transfer","source_channel":"channel-0","receiver":"%s"}}' "$MAIN_WALLET_ADDR_GAIA")"
#neutrond tx wasm execute "$wrap_and_send_contract_address" "$msg" --amount "200$ATOM_ON_NEUTRON_IBC_DENOM" --from "$MAIN_WALLET" "${ntx[@]}" | assert_success
#assert_balance_neutron "$wrap_and_send_contract_address" "untrn" "0"
#assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "2700"
#
#echo
#echo -n "Waiting 10 seconds for IBC transfer to complete"
## shellcheck disable=SC2034
#for i in $(seq 10); do
#  sleep 1
#  echo -n .
#done
#echo " done"
#
#watom_on_gaia_ibc_denom="ibc/$(printf 'transfer/channel-0/factory/%s/wATOM' "$lido_satellite_contract_address" \
#  | sha256sum - | awk '{print $1}' | tr '[:lower:]' '[:upper:]')"
#echo
#assert_balance_neutron "$wrap_and_send_contract_address" "untrn" "1000"
#assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "2700"
#assert_balance_gaia "$MAIN_WALLET_ADDR_GAIA" "$watom_on_gaia_ibc_denom" "200"
