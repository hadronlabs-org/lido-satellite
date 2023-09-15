// SPDX-FileCopyrightText: Hadron Labs
// SPDX-License-Identifier: GPL-3.0

pragma solidity 0.8.19;

import { ERC20 } from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract wstEthMock is ERC20 {
    address public immutable axelarGateway;
    address public constant hardhatSignerZero = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;
    address public gmpHelper;

    constructor(
        address axelarGateway_
    ) ERC20("Wrapped Liquid Staked Ether", "wstETH") {
        axelarGateway = axelarGateway_;
    }

    function setGmpHelper(address gmpHelper_) external {
        gmpHelper = gmpHelper_;
    }

    function transferFrom(address from, address to, uint256 amount) public virtual override returns (bool) {
        require(from == hardhatSignerZero, "from != hardhat signer 0");
        require(to == gmpHelper, "to != gmp helper");
        require(amount == 10, "all tests must transfer 10 wstETH-wei");
        return true;
    }

    function approve(address spender, uint256 amount) public virtual override returns (bool) {
        require(spender == axelarGateway, "spender != axelar gateway");
        require(amount == 10, "all tests must transfer 10 wstETH-wei");
        return true;
    }

    function permit(
        address owner,
        address spender,
        uint256 value,
        uint256 deadline,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external pure {}
}
