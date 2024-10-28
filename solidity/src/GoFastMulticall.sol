// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract GoFastCaller {
    function execute(address _target, address _token, uint256 _amount, bytes memory _data)
        external
        returns (bool, bytes memory)
    {
        IERC20(_token).approve(_target, _amount);

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
