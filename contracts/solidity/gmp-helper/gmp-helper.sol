// SPDX-License-Identifier: Apache-2
pragma solidity ^0.8.9;

import "@axelar-network/axelar-gmp-sdk-solidity/contracts/interfaces/IAxelarGateway.sol";
import "@axelar-network/axelar-gmp-sdk-solidity/contracts/interfaces/IAxelarGasService.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract GmpHelper {
    IAxelarGasService public immutable gasService;
    IAxelarGateway public immutable gateway;
    IERC20 public immutable wstEth;
    string public lidoSatellite;

    string public constant DESTINATION_CHAIN = "neutron";
    string public constant WSTETH_SYMBOL = "wstETH";

    constructor(
        address axelarGateway_,
        address axelarGasReceiver_,
        address wstEth_,
        string memory lidoSatellite_
    ) {
        gasService = IAxelarGasService(axelarGasReceiver_);
        gateway = IAxelarGateway(axelarGateway_);
        wstEth = IERC20(wstEth_);
        lidoSatellite = lidoSatellite_;
    }

    function send(
        string calldata receiver,
        uint256 amount
    ) external payable {
        // 1. withdraw wstETH from caller and approve it for Axelar Gateway.
        // Gateway will attempt to transfer funds from address(this), hence we
        // are forced to withdraw them from caller account first.
        wstEth.transferFrom(msg.sender, address(this), amount);
        wstEth.approve(address(gateway), amount);

        // 2. Generate GMP payload
        bytes memory payload = _encodeGmpPayload(receiver);

        // 3. Pay for gas
        gasService.payNativeGasForContractCallWithToken{value: msg.value}(
            address(this),
            DESTINATION_CHAIN,
            lidoSatellite,
            payload,
            WSTETH_SYMBOL,
            amount,
            msg.sender
        );

        // 4. Make GMP call
        gateway.callContractWithToken(
            DESTINATION_CHAIN,
            lidoSatellite,
            payload,
            WSTETH_SYMBOL,
            amount
        );
    }

    function _encodeGmpPayload(
        string memory targetReceiver
    ) internal pure returns (bytes memory) {
        bytes memory argValues = abi.encode(
            targetReceiver
        );

        string[] memory argumentNameArray = new string[](1);
        argumentNameArray[0] = "receiver";

        string[] memory abiTypeArray = new string[](1);
        abiTypeArray[0] = "string";

        bytes memory gmpPayload;
        gmpPayload = abi.encode(
            "mint",
            argumentNameArray,
            abiTypeArray,
            argValues
        );

        return abi.encodePacked(
            bytes4(0x00000001),
            gmpPayload
        );
    }
}