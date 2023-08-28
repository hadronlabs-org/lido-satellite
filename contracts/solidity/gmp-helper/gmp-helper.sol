// SPDX-License-Identifier: Apache-2
pragma solidity ^0.8.9;

import "@axelar-network/axelar-gmp-sdk-solidity/contracts/interfaces/IAxelarGateway.sol";
import "@axelar-network/axelar-gmp-sdk-solidity/contracts/interfaces/IAxelarGasService.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/IERC20Permit.sol";

interface IWSTETH is IERC20, IERC20Permit {}

/// @title GMP helper which makes it easier to call Lido Satellite on Neutron
/// @author Murad Karammaev
/// @notice Default flow (without a GMP Helper) is to:
///           1. tx approve() on wstETH contract
///           2. tx payNativeGasForContractCallWithToken() on Axelar Gas Service
///           3. tx callContractWithToken() on Axelar gateway
///         This contract simplifies it to:
///           1. tx approve() on wstETH contract
///           2. tx send() on GMP Helper
///         It is also possible to simplify it further if user wallet supports EIP-712 signing:
///           1. tx sendWithPermit() on GMP Helper
contract GmpHelper {
    IAxelarGasService public immutable gasService;
    IAxelarGateway public immutable gateway;
    IWSTETH public immutable wstEth;
    string public lidoSatellite;

    string public constant DESTINATION_CHAIN = "neutron";
    string public constant WSTETH_SYMBOL = "wstETH";

    /// @notice Construct GMP Helper
    /// @param axelarGateway_ Address of Axelar Gateway contract
    /// @param axelarGasReceiver_ Address of Axelar Gas Service contract
    /// @param wstEth_ Address of Wrapped Liquid Staked Ether contract
    /// @param lidoSatellite_ Address of Lido Satellite contract on Neutron
    constructor(
        address axelarGateway_,
        address axelarGasReceiver_,
        address wstEth_,
        string memory lidoSatellite_
    ) {
        gasService = IAxelarGasService(axelarGasReceiver_);
        gateway = IAxelarGateway(axelarGateway_);
        wstEth = IWSTETH(wstEth_);
        lidoSatellite = lidoSatellite_;
    }

    /// @notice Send `amount` of wstETH to `receiver` on Neutron.
    ///         Requires allowance on wstETH contract.
    ///         Requires gas fee in ETH.
    /// @param receiver Address on Neutron which shall receive canonical wstETH
    /// @param amount Amount of wstETH-wei to send to `receiver`
    function send(
        string calldata receiver,
        uint256 amount
    ) external payable {
        _send(receiver, amount);
    }

    /// @notice Send `amount` of wstETH to `receiver` on Neutron, using EIP-2612 permit.
    ///         Requires gas fee in ETH.
    /// @param receiver Address on Neutron which shall receive canonical wstETH
    /// @param amount Amount of wstETH-wei to send to `receiver`
    /// @param deadline EIP-2612 permit signature deadline
    /// @param v Value `v` of EIP-2612 permit signature
    /// @param r Value `r` of EIP-2612 permit signature
    /// @param s Value `s` of EIP-2612 permit signature
    function sendWithPermit(
        string calldata receiver,
        uint256 amount,
        uint256 deadline,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external payable {
        wstEth.permit(msg.sender, address(this), amount, deadline, v, r, s);
        _send(receiver, amount);
    }

    function _send(
        string calldata receiver,
        uint256 amount
    ) internal {
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
