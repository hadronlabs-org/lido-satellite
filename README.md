# Lido Satellite

## Overview

This contract is designed to enforce a canonical denom for any kind of bridged wsteth fund,
since there may be different ways to bridge it, and having a single interoperable canonical demon
will make operations on them much easier.

## Flow

Users can mint canonical denom via sending bridged funds to the contract with `ExecuteMsg::Mint`.
To get funds back, users can simply send canonical funds to the contract with `ExecuteMsg::Burn`.
Optionally, users may specify both in `ExecuteMsg::Mint` and `ExecuteMsg::Burn` the receiver
of coins, if they are willing to receive them on some other address.

## Deployment

This contract utilizes tokenfactory in order to mint canonical funds. Tokenfactory denom is created
the moment a contract is instantiated, that means one will have to supply 1000000untrn (this amount
was required at the moment of writing this document, the amount may change through governance process)
along with instantiate message. Incomplete example of instantiating this contract via CLI looks
like this:

```bash
neutrond tx wasm instantiate 42                                \
  '{"bridged_denom":"ibc/12345","canonical_subdenom":"wsteth"}' \
  --amount 1000000untrn
```

This example assumes bridged denomination is `ibc/12345`. Resulting canonical
denomination will be of form `factory/$contract_address/wsteth`,
where `$contract_address` is the address of instantiated contract, and `wsteth`
is a string value `canonical_subdenom` we have just set in the instantiate message.

## Tests

In order to run integration tests:

1. Go to neutron's folder and run `make init && make start-rly`, wait for chain and hermes to launch
2. Get back to lido-satellite's folder and run `make build`
3. Run `./integration_test.bash` and wait until it finishes

It is expected to print

```
[OK] Main wallet has lost 3000 ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2
[OK] Second wallet has earned 500 ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2

INTEGRATION TESTS SUCCEDED
```

If it doesn't, something is really wrong.

# GMP Helper

## Overview

This contract is a little helper contract which simplifies user intercation with wstETH bridge,
requiring them to sign only 2 transactions on Ethereum instead of 3.

## Deployment

#### 1. Set Lido Satellite address

There is a hardcoded `LIDO_SATELLITE` constant string in `gmp-helper.sol`. It is hardcoded in order
to save gas and to avoid using contract storage. You have to update it with a corrrect value before
deploying a contract. You can retrieve Lido Satellite address from Neutron documentation,
there is a page for [testnet](https://docs.neutron.org/deployment/testnet#bridge)
and a page for [mainnet](https://docs.neutron.org/deployment/mainnet#bridge).

#### 2. Set Etherscan API key and network private keys

Create `.env` file and fill it with following contents:

```
ETHERSCAN_API_KEY=""
GOERLI_PRIVATE_KEY=""
ETHEREUM_PRIVATE_KEY=""
```

#### 3. Deploy GMP Helper

###### Testnet

```bash
env                                                               \
  AXELAR_GATEWAY="0xe432150cce91c13a887f7D836923d5597adD8E31"     \
  AXELAR_GAS_SERVICE="0xbE406F0189A0B4cf3A05C286473D23791Dd44Cc6" \
  WST_ETH="0x6320cD32aA674d2898A68ec82e869385Fc5f7E2f"            \
  npx hardhat run --network goerli scripts/deploy_gmp_helper.js
```

###### Mainnet

```bash
env                                                               \
  AXELAR_GATEWAY="0x4F4495243837681061C4743b74B3eEdf548D56A5"     \
  AXELAR_GAS_SERVICE="0x2d5d7d31F671F86C782533cc367F14109a082712" \
  WST_ETH="0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0"            \
  npx hardhat run --network ethereum scripts/deploy_gmp_helper.js
```

#### 4. Deploy Ossifiable Proxy

###### Testnet

```bash
env                                                                      \
  IMPLEMENTATION="<insert GMP Helper address obtained at previous step>" \
  ADMIN="<insert address which should be the admin of the contract>"     \
  npx hardhat run --network goerli scripts/deploy_ossifiable_proxy.js
```

###### Mainnet

```bash
env                                                                      \
  IMPLEMENTATION="<insert GMP Helper address obtained at previous step>" \
  ADMIN="<insert address which should be the admin of the contract>"     \
  npx hardhat run --network ethereum scripts/deploy_ossifiable_proxy.js
```
