.PHONY: schema test clippy proto-gen build fmt

schema:
	@find contracts/cosmwasm/* -maxdepth 0 -type d \( ! -name . \) -exec bash -c "cd '{}' && cargo schema" \;

test:
	@cargo test

clippy:
	@cargo clippy --all --all-targets -- -D warnings

fmt:
	@cargo fmt -- --check

check_contracts:
	@cargo install cosmwasm-check
	@cosmwasm-check --available-capabilities iterator,staking,stargate,neutron artifacts/*.wasm

compile:
	@scripts/build_release.sh

build: schema clippy fmt test compile check_contracts

lint-sol:
	@npx solhint contracts/solidity/gmp-helper/gmp-helper.sol

build-sol:
	@npx hardhat compile

test-sol: build-sol
	@npx hardhat test
