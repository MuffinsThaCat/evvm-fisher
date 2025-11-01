// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../interfaces/IEVVMCore.sol";
import "../libraries/MathLib.sol";

/**
 * @title FisherWithFees
 * @notice Complete Fisher with fee collection and gas reimbursement
 * @dev Extends OptimizedFisher with proper economic model
 * 
 * Fee Model:
 * - Users deposit MATE tokens to Fisher contract
 * - Fisher deducts relayer fee from deposits
 * - Remaining balance forwarded to EVVM Core
 * - Gas savings benefit users (86% reduction)
 */
contract FisherWithFees {
    using MathLib for uint256;
    
    // ============ Immutable State ============
    
    IEVVMCore public immutable evvmCore;
    address public immutable operator;
    address public immutable mateToken; // MATE ERC20 token
    
    // ============ Mutable State ============
    
    /// @notice User deposits (in MATE tokens)
    mapping(address => uint256) public deposits;
    
    /// @notice Relayer fee in basis points (1 = 0.01%)
    uint256 public relayerFeeBps;
    
    /// @notice Accumulated relayer fees
    uint256 public accumulatedFees;
    
    /// @notice Minimum batch size
    uint256 public minBatchSize;
    
    /// @notice Batch counter
    uint256 public batchCounter;
    
    /// @notice Emergency pause
    bool public paused;
    
    // ============ Events ============
    
    event Deposited(address indexed user, uint256 amount);
    event Withdrawn(address indexed user, uint256 amount);
    event FeeCollected(address indexed user, uint256 amount);
    
    event BatchSubmitted(
        uint256 indexed batchId,
        uint256 operationCount,
        uint256 gasUsed,
        uint256 gasSaved,
        uint256 feesCollected,
        uint256 timestamp
    );
    
    // ============ Errors ============
    
    error Unauthorized();
    error Paused();
    error InsufficientDeposit();
    error InvalidBatchSize();
    error ArrayLengthMismatch();
    
    // ============ Modifiers ============
    
    modifier onlyOperator() {
        if (msg.sender != operator) revert Unauthorized();
        _;
    }
    
    modifier whenNotPaused() {
        if (paused) revert Paused();
        _;
    }
    
    // ============ Constructor ============
    
    constructor(
        address _evvmCore,
        address _mateToken,
        uint256 _relayerFeeBps,
        uint256 _minBatchSize
    ) {
        require(_evvmCore != address(0), "Invalid EVVM");
        require(_mateToken != address(0), "Invalid MATE");
        require(_relayerFeeBps <= 1000, "Fee too high"); // Max 10%
        
        evvmCore = IEVVMCore(_evvmCore);
        mateToken = _mateToken;
        operator = msg.sender;
        relayerFeeBps = _relayerFeeBps;
        minBatchSize = _minBatchSize;
    }
    
    // ============ Deposit Functions ============
    
    /**
     * @notice Deposit MATE tokens for transaction fees
     * @dev Users must approve Fisher contract first, then deposit
     * @param amount Amount of MATE tokens to deposit
     */
    function deposit(uint256 amount) external {
        // Transfer MATE tokens from user to Fisher
        (bool success, bytes memory data) = mateToken.call(
            abi.encodeWithSignature("transferFrom(address,address,uint256)", msg.sender, address(this), amount)
        );
        require(success && (data.length == 0 || abi.decode(data, (bool))), "MATE transfer failed");
        
        deposits[msg.sender] += amount;
        emit Deposited(msg.sender, amount);
    }
    
    /**
     * @notice Withdraw unused deposits
     * @param amount Amount to withdraw
     */
    function withdraw(uint256 amount) external {
        if (deposits[msg.sender] < amount) revert InsufficientDeposit();
        
        deposits[msg.sender] -= amount;
        emit Withdrawn(msg.sender, amount);
        
        // Transfer MATE tokens back to user
        (bool success, bytes memory data) = mateToken.call(
            abi.encodeWithSignature("transfer(address,uint256)", msg.sender, amount)
        );
        require(success && (data.length == 0 || abi.decode(data, (bool))), "MATE transfer failed");
    }
    
    // ============ Core Fisher Function ============
    
    /**
     * @notice Submit batch with automatic fee collection
     * @param payments Array of payments (amounts include relayer fee)
     * @param signatures Array of signatures
     */
    function submitBatchWithFees(
        IEVVMCore.Payment[] calldata payments,
        bytes[] calldata signatures
    ) external whenNotPaused returns (bool[] memory results) {
        uint256 n = payments.length;
        
        if (n != signatures.length) revert ArrayLengthMismatch();
        if (n < minBatchSize) revert InvalidBatchSize();
        
        uint256 gasStart = gasleft();
        uint256 totalFeesCollected = 0;
        
        // Calculate optimal chunk size
        uint256 chunkSize = n.williamsChunkSize();
        
        // Allocate memory for chunk processing
        IEVVMCore.Payment[] memory chunk = new IEVVMCore.Payment[](chunkSize);
        bytes[] memory sigChunk = new bytes[](chunkSize);
        results = new bool[](n);
        
        // Process in memory-efficient chunks
        for (uint256 i = 0; i < n; i += chunkSize) {
            uint256 end = (i + chunkSize).min(n);
            uint256 currentChunkSize = end - i;
            
            // Load and process chunk
            for (uint256 j = 0; j < currentChunkSize; j++) {
                IEVVMCore.Payment memory payment = payments[i + j];
                
                // Calculate and collect fee
                uint256 fee = (payment.amount * relayerFeeBps) / 10000;
                uint256 netAmount = payment.amount - fee;
                
                // Check user has sufficient deposit
                if (deposits[payment.from] < payment.amount) {
                    revert InsufficientDeposit();
                }
                
                // Deduct from deposit and collect fee
                deposits[payment.from] -= payment.amount;
                totalFeesCollected += fee;
                
                // Store modified payment (net amount)
                chunk[j] = IEVVMCore.Payment({
                    from: payment.from,
                    to: payment.to,
                    amount: netAmount,
                    priorityFlag: payment.priorityFlag,
                    nonce: payment.nonce
                });
                
                sigChunk[j] = signatures[i + j];
                
                emit FeeCollected(payment.from, fee);
            }
            
            // Create temporary arrays for EVVM
            IEVVMCore.Payment[] memory tempPayments = new IEVVMCore.Payment[](currentChunkSize);
            bytes[] memory tempSignatures = new bytes[](currentChunkSize);
            
            for (uint256 j = 0; j < currentChunkSize; j++) {
                tempPayments[j] = chunk[j];
                tempSignatures[j] = sigChunk[j];
            }
            
            // Submit to EVVM Core
            bool[] memory chunkResults = evvmCore.payMultiple(tempPayments, tempSignatures);
            
            // Store results
            for (uint256 j = 0; j < currentChunkSize; j++) {
                results[i + j] = chunkResults[j];
            }
        }
        
        // Accumulate fees
        accumulatedFees += totalFeesCollected;
        
        uint256 gasUsed = gasStart - gasleft();
        uint256 gasSaved = _calculateGasSaved(n, gasUsed);
        
        emit BatchSubmitted(
            ++batchCounter,
            n,
            gasUsed,
            gasSaved,
            totalFeesCollected,
            block.timestamp
        );
        
        return results;
    }
    
    // ============ View Functions ============
    
    function calculateChunkSize(uint256 batchSize) external pure returns (uint256) {
        return batchSize.williamsChunkSize();
    }
    
    function estimateGas(uint256 batchSize) external pure returns (
        uint256 estimatedGas,
        uint256 estimatedSavings
    ) {
        estimatedGas = batchSize * 14_000;
        estimatedSavings = (batchSize * 100_000) - estimatedGas;
    }
    
    function estimateFee(uint256 amount) external view returns (uint256) {
        return (amount * relayerFeeBps) / 10000;
    }
    
    function _calculateGasSaved(uint256 n, uint256 actualGas) internal pure returns (uint256) {
        uint256 traditionalGas = n * 100_000;
        return traditionalGas > actualGas ? traditionalGas - actualGas : 0;
    }
    
    // ============ Admin Functions ============
    
    function setRelayerFee(uint256 newFeeBps) external onlyOperator {
        require(newFeeBps <= 1000, "Fee too high");
        relayerFeeBps = newFeeBps;
    }
    
    function withdrawFees(address to, uint256 amount) external onlyOperator {
        require(amount <= accumulatedFees, "Insufficient fees");
        accumulatedFees -= amount;
        
        // Transfer MATE token fees to operator
        (bool success, bytes memory data) = mateToken.call(
            abi.encodeWithSignature("transfer(address,uint256)", to, amount)
        );
        require(success && (data.length == 0 || abi.decode(data, (bool))), "MATE transfer failed");
    }
    
    function togglePause() external onlyOperator {
        paused = !paused;
    }
}
