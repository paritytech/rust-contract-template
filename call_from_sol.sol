// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.0;

interface IRustContract {
    function fibonacci(uint32) external pure returns (uint32);
}

contract CallRust {
    function fibonacci(uint32 n) public pure returns (uint32) {
        if (n == 0) {
            return 0;
        } else if (n == 1) {
            return 1;
        } else {
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
    }

    function fibonacciRust(uint32 n, IRustContract rustLib) external pure returns (uint32) {
        try rustLib.fibonacci(n) returns (uint32 result) {
            return result;
        } catch (bytes memory) {
            revert("calling into rust failed");
        }
    }
}
