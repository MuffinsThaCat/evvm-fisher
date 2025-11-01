// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title IEVVMCore
 * @notice Interface for EVVM Core Contract
 * @dev Based on EVVM documentation - update with actual contract ABI when available
 */
interface IEVVMCore {
    
    // ============ Structs ============
    
    struct Payment {
        address from;
        address to;
        uint256 amount;
        bool priorityFlag;  // true = asynchronous nonce, false = synchronous
        uint256 nonce;      // Used when priorityFlag = true
    }
    
    // ============ Events ============
    
    event PaymentProcessed(
        address indexed from,
        address indexed to,
        uint256 amount,
        uint256 timestamp
    );
    
    event BatchProcessed(
        uint256 indexed batchId,
        uint256 successCount,
        uint256 totalCount,
        uint256 timestamp
    );
    
    event DispersalProcessed(
        address indexed sender,
        uint256 recipientCount,
        uint256 totalAmount,
        uint256 timestamp
    );
    
    // ============ Core Payment Functions ============
    
    /**
     * @notice Execute single payment with automatic staker detection
     * @param from Sender address
     * @param to Recipient address
     * @param amount Token amount
     * @param priorityFlag false = synchronous nonce, true = asynchronous
     * @param signature EIP-191 signature
     */
    function pay(
        address from,
        address to,
        uint256 amount,
        bool priorityFlag,
        bytes calldata signature
    ) external;
    
    /**
     * @notice Execute multiple payments in batch
     * @param payments Array of payment structures
     * @param signatures Array of EIP-191 signatures
     * @return results Array of success booleans for each payment
     */
    function payMultiple(
        Payment[] calldata payments,
        bytes[] calldata signatures
    ) external returns (bool[] memory results);
    
    /**
     * @notice Distribute from one sender to multiple recipients
     * @param sender Source address
     * @param recipients Array of recipient addresses
     * @param amounts Array of amounts (must match recipients length)
     * @param signature EIP-191 signature
     */
    function dispersePay(
        address sender,
        address[] calldata recipients,
        uint256[] calldata amounts,
        bytes calldata signature
    ) external;
    
    // ============ Contract Payment Functions ============
    
    /**
     * @notice Contract-to-address payment (no approval needed)
     * @param from Contract address
     * @param to Recipient address
     * @param amount Token amount
     */
    function caPay(
        address from,
        address to,
        uint256 amount
    ) external;
    
    /**
     * @notice Contract distribution to multiple addresses
     * @param sender Contract address
     * @param recipients Array of recipient addresses
     * @param amounts Array of amounts
     */
    function disperseCaPay(
        address sender,
        address[] calldata recipients,
        uint256[] calldata amounts
    ) external;
    
    // ============ Getter Functions ============
    
    /**
     * @notice Get principal token balance
     * @param account Address to query
     * @return balance Principal token balance
     */
    function principalBalanceOf(address account) external view returns (uint256 balance);
    
    /**
     * @notice Get reward balance
     * @param account Address to query
     * @return balance Reward token balance
     */
    function rewardBalanceOf(address account) external view returns (uint256 balance);
    
    /**
     * @notice Get current nonce for address
     * @param account Address to query
     * @return nonce Current nonce value
     */
    function getNonce(address account) external view returns (uint256 nonce);
    
    /**
     * @notice Check if address is staker
     * @param account Address to check
     * @return isStaker True if address has staked principal tokens
     */
    function isStaker(address account) external view returns (bool isStaker);
    
    /**
     * @notice Get current era number
     * @return era Current era for rewards
     */
    function getCurrentEra() external view returns (uint256 era);
}
