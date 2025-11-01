// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title MathLib
 * @notice Gas-optimized mathematical operations for Williams compression
 * @dev Implements sqrt and log2 using Babylonian method and bit manipulation
 */
library MathLib {
    
    /**
     * @notice Calculate integer square root using Babylonian method
     * @dev Optimized for gas efficiency, uses bit shifting
     * @param x Input number
     * @return y Square root of x
     */
    function sqrt(uint256 x) internal pure returns (uint256 y) {
        if (x == 0) return 0;
        
        // Initial guess using bit length
        uint256 z = (x + 1) / 2;
        y = x;
        
        // Babylonian method iterations (converges quickly)
        while (z < y) {
            y = z;
            z = (x / z + z) / 2;
        }
    }
    
    /**
     * @notice Calculate log base 2 (integer)
     * @dev Simple bit-counting approach for gas efficiency
     * @param x Input number
     * @return Most significant bit position
     */
    function log2(uint256 x) internal pure returns (uint256) {
        require(x > 0, "MathLib: log2 of zero");
        
        uint256 result = 0;
        
        if (x >= 2**128) { x >>= 128; result += 128; }
        if (x >= 2**64) { x >>= 64; result += 64; }
        if (x >= 2**32) { x >>= 32; result += 32; }
        if (x >= 2**16) { x >>= 16; result += 16; }
        if (x >= 2**8) { x >>= 8; result += 8; }
        if (x >= 2**4) { x >>= 4; result += 4; }
        if (x >= 2**2) { x >>= 2; result += 2; }
        if (x >= 2**1) { result += 1; }
        
        return result;
    }
    
    /**
     * @notice Calculate Williams chunk size: sqrt(n) * log2(n)
     * @dev Optimal memory allocation for batch processing
     * @param n Number of elements
     * @return Optimal chunk size for Williams compression
     */
    function williamsChunkSize(uint256 n) internal pure returns (uint256) {
        if (n <= 1) return n;
        return sqrt(n) * log2(n);
    }
    
    /**
     * @notice Return minimum of two numbers
     * @param a First number
     * @param b Second number
     * @return Minimum value
     */
    function min(uint256 a, uint256 b) internal pure returns (uint256) {
        return a < b ? a : b;
    }
    
    /**
     * @notice Return maximum of two numbers
     * @param a First number
     * @param b Second number
     * @return Maximum value
     */
    function max(uint256 a, uint256 b) internal pure returns (uint256) {
        return a > b ? a : b;
    }
}
