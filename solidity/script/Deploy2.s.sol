// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {FastTransferGateway} from "../src/FastTransferGateway.sol";

contract DeployScript is Script {
    address public constant USDC_ABRITRUM = 0xaf88d065e77c8cC2239327C5EDb3A432268e5831;
    address public constant PERMIT2_ABRITRUM = 0x000000000022D473030F116dDEE9F6B43aC78BA3;
    address public constant MAILBOX_ABRITRUM = 0x979Ca5202784112f4738403dBec5D0F3B9daabB9;

    address public constant USDC_BASE = 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913;
    address public constant PERMIT2_BASE = 0x000000000022D473030F116dDEE9F6B43aC78BA3;
    address public constant MAILBOX_BASE = 0xeA87ae93Fa0019a82A727bfd3eBd1cFCa8f64f1D;

    address public owner = 0x56Ca414d41CD3C1188A4939b0D56417dA7Bb6DA2;

    function run(uint32 chainID) public {
        (address usdc, address permit2, address mailbox) = _getInitValues(chainID);

        vm.startBroadcast();

        FastTransferGateway gatewayImpl = new FastTransferGateway();

        ERC1967Proxy gatewayProxy = new ERC1967Proxy(
            address(gatewayImpl),
            abi.encodeWithSignature(
                "initialize(uint32,address,address,address,address)", chainID, owner, usdc, mailbox, permit2
            )
        );

        console.log("Gateway deployed at", address(gatewayProxy));

        vm.stopBroadcast();
    }

    function _getInitValues(uint32 chainID) internal pure returns (address, address, address) {
        if (chainID == 42161) {
            return (USDC_ABRITRUM, PERMIT2_ABRITRUM, MAILBOX_ABRITRUM);
        } else if (chainID == 8453) {
            return (USDC_BASE, PERMIT2_BASE, MAILBOX_BASE);
        }

        revert("Invalid chain ID");
    }
}
