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
ASTROPORT_ROUTER_PATH="artifacts/mock_astroport_router.wasm"
WRAP_AND_SEND_PATH="artifacts/wrap_and_send.wasm"
SENTINEL_PATH="artifacts/wrap_and_send_sentinel.wasm"
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

select_attr() {
  printf '.logs[0].events[] | select(.type == "%s").attributes[] | select(.key == "%s").value' "$1" "$2"
}

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

assert_no_funds_neutron() {
  local amount
  amount="$(neutrond query bank balances "$1" "${nq[@]}" | jq -r '.balances')"
  if ! [ "$amount" = '[]' ]; then
    echo "[FAIL] $1 has funds while it shouldn't: $amount"
    exit 1
  fi
  echo "[OK] $1 has no funds"
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
while ! [[ "$(get_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "$ATOM_ON_NEUTRON_IBC_DENOM")" -ge 20000 ]]; do
  echo -n "."
  ((attempts=attempts-1)) || {
    echo "seems like IBC transfer failed" 1>&2
    exit 1
  }
  sleep 0.1
done
while ! [[ "$(get_balance_neutron "$SECOND_WALLET_ADDR_NEUTRON" "$ATOM_ON_NEUTRON_IBC_DENOM")" -ge 10000 ]]; do
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

echo
lido_satellite_code_id="$(neutrond tx wasm store "$LIDO_SATELLITE_PATH" --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | jq -r "$(select_attr "store_code" "code_id")")"
echo "Lido Satellite Code ID: $lido_satellite_code_id"
msg="$(printf '{"bridged_denom":"%s","canonical_subdenom":"wATOM"}' "$ATOM_ON_NEUTRON_IBC_DENOM")"
lido_satellite_contract_address="$(neutrond tx wasm instantiate "$lido_satellite_code_id" "$msg" --amount 1000000untrn --no-admin --label lido_satellite --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | jq -r "$(select_attr "instantiate" "_contract_address")")"
echo "Lido Satellite Contract address: $lido_satellite_contract_address"

echo
echo "Mint 1000 wATOM to main account"
neutrond tx wasm execute "$lido_satellite_contract_address" '{"mint":{}}' --amount "1000$ATOM_ON_NEUTRON_IBC_DENOM" --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | assert_success
assert_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "factory/$lido_satellite_contract_address/wATOM" "1000"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "1000"

echo
echo "Mint 2000 more wATOM to main account"
neutrond tx wasm execute "$lido_satellite_contract_address" '{"mint":{}}' --amount "2000$ATOM_ON_NEUTRON_IBC_DENOM" --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | assert_success
assert_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "factory/$lido_satellite_contract_address/wATOM" "3000"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "3000"

echo
echo "Mint 1500 wATOM to second account"
neutrond tx wasm execute "$lido_satellite_contract_address" '{"mint":{}}' --amount "1500$ATOM_ON_NEUTRON_IBC_DENOM" --from "$SECOND_WALLET" "${ntx[@]}" | wait_ntx | assert_success
assert_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "factory/$lido_satellite_contract_address/wATOM" "3000"
assert_balance_neutron "$SECOND_WALLET_ADDR_NEUTRON" "factory/$lido_satellite_contract_address/wATOM" "1500"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "4500"

echo
echo "Burn 200 wATOM from main account and send $ATOM_ON_NEUTRON_IBC_DENOM to second account"
msg="$(printf '{"burn":{"receiver":"%s"}}' "$SECOND_WALLET_ADDR_NEUTRON")"
neutrond tx wasm execute "$lido_satellite_contract_address" "$msg" --amount "200factory/$lido_satellite_contract_address/wATOM" --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | assert_success
assert_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "factory/$lido_satellite_contract_address/wATOM" "2800"
assert_balance_neutron "$SECOND_WALLET_ADDR_NEUTRON" "factory/$lido_satellite_contract_address/wATOM" "1500"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "4300"

echo
echo "Send 300 wATOM from main account to second account"
neutrond tx bank send "$MAIN_WALLET" "$SECOND_WALLET_ADDR_NEUTRON" "300factory/$lido_satellite_contract_address/wATOM" "${ntx[@]}" | wait_ntx | assert_success
assert_balance_neutron "$MAIN_WALLET_ADDR_NEUTRON" "factory/$lido_satellite_contract_address/wATOM" "2500"
assert_balance_neutron "$SECOND_WALLET_ADDR_NEUTRON" "factory/$lido_satellite_contract_address/wATOM" "1800"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "4300"

echo
echo "Burn 1800 wATOM from second account"
neutrond tx wasm execute "$lido_satellite_contract_address" '{"burn":{}}' --amount "1800factory/$lido_satellite_contract_address/wATOM" --from "$SECOND_WALLET" "${ntx[@]}" | wait_ntx | assert_success
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

echo
astroport_router_code_id="$(neutrond tx wasm store "$ASTROPORT_ROUTER_PATH" --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | jq -r "$(select_attr "store_code" "code_id")")"
echo "Astroport Router Code ID: $astroport_router_code_id"
msg="$(printf '{"offer_denom":"%s","ask_denom":"untrn"}' "factory/$lido_satellite_contract_address/wATOM")"
astroport_router_contract_address="$(neutrond tx wasm instantiate "$astroport_router_code_id" "$msg" --amount 100000untrn --no-admin --label mock_astroport_router --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | jq -r "$(select_attr "instantiate" "_contract_address")")"
echo "Astroport Router Contract address: $astroport_router_contract_address"
wrap_and_send_code_id="$(neutrond tx wasm store "$WRAP_AND_SEND_PATH" --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | jq -r "$(select_attr "store_code" "code_id")")"
echo "Wrap and Send Code ID: $wrap_and_send_code_id"
msg="$(printf '{"lido_satellite":"%s","astroport_router":"%s"}' "$lido_satellite_contract_address" "$astroport_router_contract_address")"
wrap_and_send_contract_address="$(neutrond tx wasm instantiate "$wrap_and_send_code_id" "$msg" --no-admin --label wrap_and_send --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | jq -r "$(select_attr "instantiate" "_contract_address")")"
echo "Wrap and Send Contract address: $wrap_and_send_contract_address"
sentinel_code_id="$(neutrond tx wasm store "$SENTINEL_PATH" --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | jq -r "$(select_attr "store_code" "code_id")")"
echo "Sentinel Code ID: $sentinel_code_id"
msg="$(printf '{"wrap_and_send":"%s"}' "$wrap_and_send_contract_address")"
sentinel_contract_address="$(neutrond tx wasm instantiate "$sentinel_code_id" "$msg" --no-admin --label sentinel --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | jq -r "$(select_attr "instantiate" "_contract_address")")"
echo "Sentinel Contract address: $sentinel_contract_address"

# a very easy way in a shell script to create a random account is to spawn some unused contract :)))
refund_address="$(neutrond tx wasm instantiate "$sentinel_code_id" "$msg" --no-admin --label refund --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | jq -r "$(select_attr "instantiate" "_contract_address")")"
echo "Refund address: $refund_address"

echo
echo "Ideal scenario: mint 1000wATOM, swap 300wATOM for IBC fee (and receive exactly as many as needed), send 700wATOM to Gaia"
msg="$(printf '{
  "wrap_and_send": {
    "source_port": "transfer",
    "source_channel": "channel-0",
    "receiver": "%s",
    "amount_to_swap_for_ibc_fee": "300",
    "ibc_fee_denom": "untrn",
    "astroport_swap_operations": [{"native_swap":{"offer_denom": "%s", "ask_denom":"untrn"}}],
    "refund_address": "%s"
  }
}' "$MAIN_WALLET_ADDR_GAIA" "factory/$lido_satellite_contract_address/wATOM" "$refund_address")"
neutrond tx wasm execute "$sentinel_contract_address" "$msg" --amount "1000$ATOM_ON_NEUTRON_IBC_DENOM" --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | assert_success
assert_balance_neutron "$refund_address" "untrn" "0"
assert_balance_neutron "$astroport_router_contract_address" "factory/$lido_satellite_contract_address/wATOM" "300"
assert_balance_neutron "$astroport_router_contract_address" "untrn" "98000"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "3500"
assert_no_funds_neutron "$wrap_and_send_contract_address"
assert_no_funds_neutron "$sentinel_contract_address"

echo -n "Waiting 10 seconds for IBC transfer to complete"
# shellcheck disable=SC2034
for i in $(seq 10); do
  sleep 1
  echo -n .
done
echo " done"

watom_on_gaia_ibc_denom="ibc/$(printf 'transfer/channel-0/factory/%s/wATOM' "$lido_satellite_contract_address" \
  | sha256sum - | awk '{print $1}' | tr '[:lower:]' '[:upper:]')"
assert_balance_neutron "$refund_address" "untrn" "1000"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "3500"
assert_balance_gaia "$MAIN_WALLET_ADDR_GAIA" "$watom_on_gaia_ibc_denom" "700"
assert_no_funds_neutron "$wrap_and_send_contract_address"
assert_no_funds_neutron "$sentinel_contract_address"

echo
echo "Sunny day scenario: mint 1500wATOM, swap 400wATOM for IBC fee (and receive more than needed), send 1100wATOM to Gaia"
msg="$(printf '{
  "wrap_and_send": {
    "source_port": "transfer",
    "source_channel": "channel-0",
    "receiver": "%s",
    "amount_to_swap_for_ibc_fee": "400",
    "ibc_fee_denom": "untrn",
    "astroport_swap_operations": [{"native_swap":{"offer_denom": "%s", "ask_denom":"untrn"}}],
    "refund_address": "%s"
  }
}' "$MAIN_WALLET_ADDR_GAIA" "factory/$lido_satellite_contract_address/wATOM" "$refund_address")"
neutrond tx wasm execute "$sentinel_contract_address" "$msg" --amount "1500$ATOM_ON_NEUTRON_IBC_DENOM" --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | assert_success
assert_balance_neutron "$refund_address" "untrn" "1334"
assert_balance_neutron "$astroport_router_contract_address" "factory/$lido_satellite_contract_address/wATOM" "700"
assert_balance_neutron "$astroport_router_contract_address" "untrn" "95666"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "5000"
assert_no_funds_neutron "$wrap_and_send_contract_address"
assert_no_funds_neutron "$sentinel_contract_address"

echo -n "Waiting 10 seconds for IBC transfer to complete"
# shellcheck disable=SC2034
for i in $(seq 10); do
  sleep 1
  echo -n .
done
echo " done"

assert_balance_neutron "$refund_address" "untrn" "2334"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "5000"
assert_balance_gaia "$MAIN_WALLET_ADDR_GAIA" "$watom_on_gaia_ibc_denom" "1800"
assert_no_funds_neutron "$wrap_and_send_contract_address"
assert_no_funds_neutron "$sentinel_contract_address"

echo
echo "Impossible scenario: mint 600wATOM, swap 200wATOM for IBC fee (and receive less than expected)"
msg="$(printf '{
  "wrap_and_send": {
    "source_port": "transfer",
    "source_channel": "channel-0",
    "receiver": "%s",
    "amount_to_swap_for_ibc_fee": "200",
    "ibc_fee_denom": "untrn",
    "astroport_swap_operations": [{"native_swap":{"offer_denom": "%s", "ask_denom":"untrn"}}],
    "refund_address": "%s"
  }
}' "$MAIN_WALLET_ADDR_GAIA" "factory/$lido_satellite_contract_address/wATOM" "$refund_address")"
neutrond tx wasm execute "$sentinel_contract_address" "$msg" --amount "600$ATOM_ON_NEUTRON_IBC_DENOM" --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | assert_success
assert_balance_neutron "$refund_address" "untrn" "2334"
assert_balance_neutron "$refund_address" "factory/$lido_satellite_contract_address/wATOM" "600"
assert_balance_neutron "$astroport_router_contract_address" "factory/$lido_satellite_contract_address/wATOM" "700"
assert_balance_neutron "$astroport_router_contract_address" "untrn" "95666"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "5600"
assert_no_funds_neutron "$wrap_and_send_contract_address"
assert_no_funds_neutron "$sentinel_contract_address"

echo
echo "Rainy day scenario: mint 500wATOM, swap 100wATOM for IBC fee (swap will fail)"
msg="$(printf '{
  "wrap_and_send": {
    "source_port": "transfer",
    "source_channel": "channel-0",
    "receiver": "%s",
    "amount_to_swap_for_ibc_fee": "100",
    "ibc_fee_denom": "untrn",
    "astroport_swap_operations": [{"native_swap":{"offer_denom": "%s", "ask_denom":"untrn"}}],
    "refund_address": "%s"
  }
}' "$MAIN_WALLET_ADDR_GAIA" "factory/$lido_satellite_contract_address/wATOM" "$refund_address")"
neutrond tx wasm execute "$sentinel_contract_address" "$msg" --amount "500$ATOM_ON_NEUTRON_IBC_DENOM" --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | assert_success
assert_balance_neutron "$refund_address" "untrn" "2334"
assert_balance_neutron "$refund_address" "factory/$lido_satellite_contract_address/wATOM" "1100"
assert_balance_neutron "$astroport_router_contract_address" "factory/$lido_satellite_contract_address/wATOM" "700"
assert_balance_neutron "$astroport_router_contract_address" "untrn" "95666"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "6100"
assert_no_funds_neutron "$wrap_and_send_contract_address"
assert_no_funds_neutron "$sentinel_contract_address"

echo
echo "Rainy day scenario: mint 200wATOM, swap 300wATOM for IBC fee (not enough tokens for swap)"
msg="$(printf '{
  "wrap_and_send": {
    "source_port": "transfer",
    "source_channel": "channel-0",
    "receiver": "%s",
    "amount_to_swap_for_ibc_fee": "300",
    "ibc_fee_denom": "untrn",
    "astroport_swap_operations": [{"native_swap":{"offer_denom": "%s", "ask_denom":"untrn"}}],
    "refund_address": "%s"
  }
}' "$MAIN_WALLET_ADDR_GAIA" "factory/$lido_satellite_contract_address/wATOM" "$refund_address")"
neutrond tx wasm execute "$sentinel_contract_address" "$msg" --amount "200$ATOM_ON_NEUTRON_IBC_DENOM" --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | assert_success
assert_balance_neutron "$refund_address" "untrn" "2334"
assert_balance_neutron "$refund_address" "factory/$lido_satellite_contract_address/wATOM" "1300"
assert_balance_neutron "$astroport_router_contract_address" "factory/$lido_satellite_contract_address/wATOM" "700"
assert_balance_neutron "$astroport_router_contract_address" "untrn" "95666"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "6300"
assert_no_funds_neutron "$wrap_and_send_contract_address"
assert_no_funds_neutron "$sentinel_contract_address"

echo
echo "Rainy day scenario: mint 400wATOM, swap 300wATOM for IBC fee, initiate IBC transfer of 100wATOM to Gaia using wrong source port"
msg="$(printf '{
  "wrap_and_send": {
    "source_port": "mistery-port",
    "source_channel": "channel-0",
    "receiver": "%s",
    "amount_to_swap_for_ibc_fee": "300",
    "ibc_fee_denom": "untrn",
    "astroport_swap_operations": [{"native_swap":{"offer_denom": "%s", "ask_denom":"untrn"}}],
    "refund_address": "%s"
  }
}' "$MAIN_WALLET_ADDR_GAIA" "factory/$lido_satellite_contract_address/wATOM" "$refund_address")"
neutrond tx wasm execute "$sentinel_contract_address" "$msg" --amount "400$ATOM_ON_NEUTRON_IBC_DENOM" --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | assert_success
assert_balance_neutron "$refund_address" "untrn" "2334"
assert_balance_neutron "$refund_address" "factory/$lido_satellite_contract_address/wATOM" "1700"
assert_balance_neutron "$astroport_router_contract_address" "factory/$lido_satellite_contract_address/wATOM" "700"
assert_balance_neutron "$astroport_router_contract_address" "untrn" "95666"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "6700"
assert_no_funds_neutron "$wrap_and_send_contract_address"
assert_no_funds_neutron "$sentinel_contract_address"

echo
echo "Rainy day scenario: mint 400wATOM, swap 300wATOM for IBC fee, initiate IBC transfer of 100wATOM to Gaia using wrong source channel"
msg="$(printf '{
  "wrap_and_send": {
    "source_port": "transfer",
    "source_channel": "channel-xxx",
    "receiver": "%s",
    "amount_to_swap_for_ibc_fee": "300",
    "ibc_fee_denom": "untrn",
    "astroport_swap_operations": [{"native_swap":{"offer_denom": "%s", "ask_denom":"untrn"}}],
    "refund_address": "%s"
  }
}' "$MAIN_WALLET_ADDR_GAIA" "factory/$lido_satellite_contract_address/wATOM" "$refund_address")"
neutrond tx wasm execute "$sentinel_contract_address" "$msg" --amount "400$ATOM_ON_NEUTRON_IBC_DENOM" --from "$MAIN_WALLET" "${ntx[@]}" | wait_ntx | assert_success
assert_balance_neutron "$refund_address" "untrn" "2334"
assert_balance_neutron "$refund_address" "factory/$lido_satellite_contract_address/wATOM" "2100"
assert_balance_neutron "$astroport_router_contract_address" "factory/$lido_satellite_contract_address/wATOM" "700"
assert_balance_neutron "$astroport_router_contract_address" "untrn" "95666"
assert_balance_neutron "$lido_satellite_contract_address" "$ATOM_ON_NEUTRON_IBC_DENOM" "7100"
assert_balance_neutron "$wrap_and_send_contract_address" "untrn" "0"
assert_no_funds_neutron "$wrap_and_send_contract_address"
assert_no_funds_neutron "$sentinel_contract_address"
