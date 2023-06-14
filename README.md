# Lido Satellite

## Overview

This contract is designed to enforce a canonical denom for any kind of bridged steth fund,
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
  '{"bridged_denom":"ibc/12345","canonical_subdenom":"steth"}' \
  --amount 1000000untrn
```

This example assumes bridged denomination is `ibc/12345`. Resulting canonical
denomination will be of form `factory/$contract_address/steth`,
where `$contract_address` is the address of instantiated contract, and `steth`
is a string value `canonical_subdenom` we have just set in the instantiate message.

## Tests

In order to run integration tests:

1. Go to neutron's folder and run `make start`, wait for chain to launch
2. Get back to lido-satellite's folder and run `make build`
3. Run `./integration_test.bash` and wait until it finishes

It is expected to print

```
[OK] Main wallet has lost 3000 uibcatom
[OK] Second wallet has earned 500 uibcatom

INTEGRATION TESTS SUCCEDED
```

If it doesn't, something is really wrong.
