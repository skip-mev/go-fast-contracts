// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {FastTransferGateway} from "../src/FastTransferGateway.sol";

contract DeployScript is Script {
    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        address owner = address(0x56Ca414d41CD3C1188A4939b0D56417dA7Bb6DA2);
        address token = address(0xaf88d065e77c8cC2239327C5EDb3A432268e5831); // USDC
        address mailbox = address(0x979Ca5202784112f4738403dBec5D0F3B9daabB9);
        address permit2 = address(0x000000000022D473030F116dDEE9F6B43aC78BA3);

        uint32 localDomain = 1;

        FastTransferGateway gatewayImpl = new FastTransferGateway();
        ERC1967Proxy gatewayProxy = new ERC1967Proxy(
            address(gatewayImpl),
            abi.encodeWithSignature(
                "initialize(uint32,address,address,address,address)",
                localDomain,
                owner,
                address(token),
                mailbox,
                address(permit2)
            )
        );

        console.log("Gateway deployed at", address(gatewayProxy));

        vm.stopBroadcast();
    }
}
