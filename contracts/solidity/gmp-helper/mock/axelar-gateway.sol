// SPDX-FileCopyrightText: Hadron Labs
// SPDX-License-Identifier: GPL-3.0

pragma solidity 0.8.19;

contract AxelarGatewayMock {
    address public gmpHelper;
    string public constant destinationContract = "neutron1aghwa8gcetlqsg46ha3esu8rqzy4k5z76v5r440ghneejzx8mwassk3x2s";

    constructor() {}

    function setGmpHelper(address gmpHelper_) external {
        gmpHelper = gmpHelper_;
    }

    function callContractWithToken(
        string calldata destinationChain,
        string calldata destinationContract_,
        bytes calldata, // payload
        string calldata symbol,
        uint256 amount
    ) external pure {
        require(keccak256(bytes("neutron")) == keccak256(bytes(destinationChain)), "destination chain != neutron");
        require(
            keccak256(bytes(destinationContract)) == keccak256(bytes(destinationContract_)),
            "destination contract != Lido Satellite"
        );
        // we ignore payload in this test, since it is quite complex to check
        require(keccak256(bytes(symbol)) == keccak256(bytes("wstETH")), "symbol != wstETH");
        require(amount == 10, "all tests must transfer 10 wstETH-wei");
    }
}
