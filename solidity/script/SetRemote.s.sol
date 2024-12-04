// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {FastTransferGateway} from "../src/FastTransferGateway.sol";
import {TypeCasts} from "../src/libraries/TypeCasts.sol";

contract DeployScript is Script {
    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        uint32 remoteDomain = 875;
        bytes32 remoteContract = hex"000000000000000000000000F7ceC3d387384bB6cE5792dAb161a65cFaCf8aB4";

        FastTransferGateway gateway = FastTransferGateway(0x80a428AEd33FeC3867850c75Ad8b6bB0Ec1270cA);

        gateway.setRemoteDomain(remoteDomain, remoteContract);

        vm.stopBroadcast();
    }
}
