// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

contract GoFastCaller is Ownable {
    using SafeERC20 for IERC20;

    address public gateway;

    constructor(address _owner) Ownable(_owner) {}

    function setGateway(address _gateway) external onlyOwner {
        gateway = _gateway;
    }

    modifier onlyGateway() {
        require(gateway != address(0), "GoFastCaller: gateway not set");
        require(msg.sender == gateway, "GoFastCaller: sender not gateway");
        _;
    }

    function execute(address _target, address _token, uint256 _amount, bytes memory _data)
        external
        onlyGateway
        returns (bool, bytes memory)
    {
        IERC20(_token).forceApprove(_target, _amount);

        (bool success, bytes memory returnData) = _target.call(_data);
        if (!success) {
            assembly {
                returndatacopy(0, 0, returndatasize())
                revert(0, returndatasize())
            }
        }
        return (success, returnData);
    }
}
