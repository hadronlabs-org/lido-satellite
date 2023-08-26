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

There are several arguments in this contract's constructor:

| name                 | description                                                                                              | testnet example                              | mainnet example                              |
|----------------------|----------------------------------------------------------------------------------------------------------|----------------------------------------------|----------------------------------------------|
| `axelarGateway_`     | Address of Axelar Gateway contract, obtained from [here](https://docs.axelar.dev/resources/mainnet)      | `0xe432150cce91c13a887f7D836923d5597adD8E31` | `0x4F4495243837681061C4743b74B3eEdf548D56A5` |
| `axelarGasReceiver_` | Address of Axelar Gas Service contract, obtained from  [here](https://docs.axelar.dev/resources/mainnet) | `0xbE406F0189A0B4cf3A05C286473D23791Dd44Cc6` | `0x2d5d7d31F671F86C782533cc367F14109a082712` |
| `wstEth_`            | Address of wstETH contract, obtained from [here](https://docs.lido.fi/deployed-contracts/)               | `0x6320cD32aA674d2898A68ec82e869385Fc5f7E2f` | `0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0` |
| `lidoSatellite_`     | Address of Lido Satellite contract deployed on Neutron                                                   | N/A                                          | N/A                                          |

In order to deploy this contract I recommend using [Remix IDE](https://remix.ethereum.org/).
