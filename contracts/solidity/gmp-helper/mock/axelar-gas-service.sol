// SPDX-FileCopyrightText: Hadron Labs
// SPDX-License-Identifier: GPL-3.0

pragma solidity 0.8.19;

contract AxelarGasServiceMock {
    address public gmpHelper;
    string public constant destinationAddress = "neutron1aghwa8gcetlqsg46ha3esu8rqzy4k5z76v5r440ghneejzx8mwassk3x2s";
    address public constant hardhatSignerZero = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;

    constructor() {}

    function setGmpHelper(address gmpHelper_) external {
        gmpHelper = gmpHelper_;
    }

    function payNativeGasForContractCallWithToken(
        address sender,
        string calldata destinationChain,
        string calldata destinationAddress_,
        bytes calldata, // payload
        string calldata symbol,
        uint256 amount,
        address refundAddress
    ) external payable {
        require(msg.value == 100, "all tests must transfer 100 ETH-wei");
        require(sender == gmpHelper, "sender != gmp helper");
        require(keccak256(bytes("neutron")) == keccak256(bytes(destinationChain)), "destination chain != neutron");
        require(
            keccak256(bytes(destinationAddress)) == keccak256(bytes(destinationAddress_)),
            "destination address != Lido Satellite"
        );
        // we ignore payload in this test, since it is quite complex to check
        require(keccak256(bytes(symbol)) == keccak256(bytes("wstETH")), "symbol != wstETH");
        require(amount == 10, "all tests must transfer 10 wstETH-wei");
        require(refundAddress == hardhatSignerZero, "refundAddress != hardhat signer 0");
    }
}
