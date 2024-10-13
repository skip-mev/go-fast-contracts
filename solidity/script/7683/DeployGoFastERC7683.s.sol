// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {GoFastERC7683} from "../../src/GoFastERC7683.sol";

contract DeployScript is Script {
    address public constant PERMIT2_ABRITRUM = 0x000000000022D473030F116dDEE9F6B43aC78BA3;

    address public owner = 0x56Ca414d41CD3C1188A4939b0D56417dA7Bb6DA2;

    function run() public {
        vm.startBroadcast();

        address gatewayAddress = 0xD594ff25a22416C0E9D2F8fD0a7166380f338977;

        GoFastERC7683 goFastERC7683Impl = new GoFastERC7683();
        ERC1967Proxy goFastERC7683Proxy = new ERC1967Proxy(
            address(goFastERC7683Impl),
            abi.encodeWithSignature("initialize(address,address,address)", msg.sender, gatewayAddress, PERMIT2_ABRITRUM)
        );

        console.log("GoFastERC7683 deployed at", address(goFastERC7683Proxy));

        vm.stopBroadcast();
    }
}
