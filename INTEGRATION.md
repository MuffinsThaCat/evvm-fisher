# Integration Guide

## Quick Start

### 1. Deploy Fisher

```bash
# Set EVVM Core address
export EVVM_CORE_ADDRESS=0x...

# Deploy
npx hardhat run scripts/deploy.ts --network sepolia
```

### 2. Submit Batches

```typescript
import { ethers } from "hardhat";

// Get fisher contract
const fisher = await ethers.getContractAt(
  "HyperOptimizedFisher",
  FISHER_ADDRESS
);

// Prepare batch
const payments = [
  {
    from: user1.address,
    to: user2.address,
    amount: ethers.parseEther("1.0"),
    priorityFlag: false,
    nonce: 0
  },
  // ... more payments
];

const signatures = [
  // EIP-191 signatures for each payment
];

// Submit optimized batch
const tx = await fisher.submitHyperOptimizedBatch(
  payments,
  signatures
);

// Check results
const receipt = await tx.wait();
console.log(`Gas used: ${receipt.gasUsed}`);
// Gas: ~9K per operation vs 100K traditional
```

### 3. Era Management

```typescript
// Advance to next era (cheap!)
await fisher.advanceFeeEra();
// Gas: ~50K vs millions for traditional

// Get fees (off-chain, free!)
const fees = await fisher.getUserFeesOffChain(
  userAddress,
  startEra,
  endEra
);
```

## Configuration

```typescript
constructor(
  address evvmCore,        // EVVM Core contract
  uint256 baseFee,         // Base fee per operation (scaled 1e18)
  uint256 feeGrowthRate,   // Fee growth per era (scaled 1e18)
  uint256 eraDuration,     // Era duration in seconds
  uint256 minBatchSize     // Minimum batch size for optimization
)
```

**Recommended settings:**
- `baseFee`: 0.001 ETH (1e15)
- `feeGrowthRate`: 5% (0.05e18)
- `eraDuration`: 86400 (1 day)
- `minBatchSize`: 10

## Gas Estimates

```typescript
// Estimate gas for batch
const [
  williamsGas,
  phiGas,
  totalSavings
] = await fisher.estimateCombinedGas(batchSize);

console.log(`Estimated: ${phiGas} gas`);
console.log(`Savings: ${totalSavings} gas`);
```

## Admin Functions

```typescript
// Update fees
await fisher.setBaseFee(newBaseFee);
await fisher.setFeeGrowthRate(newRate);

// Update batch size
await fisher.setMinBatchSize(newMin);

// Emergency pause
await fisher.togglePause();
```

## Requirements

- Solidity 0.8.19+
- Hardhat
- EVVM Core contract address
- EIP-191 signatures for transactions

## Testing

```bash
# Run all tests
npx hardhat test

# Expected: 25/25 passing, 91% savings demonstrated
```

## Support

Questions? Check:
- `/contracts/` - Source code
- `/test/` - Test examples  
- `PROPOSAL.md` - Overview
- `GAS_COMPARISON.txt` - Performance data
