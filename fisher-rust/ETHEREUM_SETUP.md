# Ethereum Contract Integration

## Your Existing Contracts

Fisher Rust now connects to your deployed contracts:

```
/evvm-optimized-fisher/contracts/fisher/FisherProduction.sol
```

## Setup

### 1. Get Your Contract Address

Your Fisher contract is deployed on Sepolia. Get the address from your deployment:

```bash
cd /Users/talzisckind/Downloads/paper/evvm-optimized-fisher
cat DEPLOYMENT_SUCCESS.md
```

Look for: `Fisher Contract: 0x...`

### 2. Configure Fisher Rust

Create `config.json`:

```json
{
  "rpc_url": "wss://eth-sepolia.g.alchemy.com/v2/YOUR_ALCHEMY_KEY",
  "fisher_address": "0xYOUR_FISHER_CONTRACT_ADDRESS",
  "evvm_core_address": "0xYOUR_EVVM_CORE_ADDRESS",
  "min_batch_size": 10,
  "max_batch_size": 1000,
  "batch_interval_ms": 5000,
  "enable_attestation": true,
  "private_key": "YOUR_PRIVATE_KEY_STARTS_WITH_0x"
}
```

### 3. Test Connection

```bash
cd fisher-rust
cargo run --example run_fisher
```

Should output:
```
🚀 Fisher Relayer started!
🔗 Connecting to Ethereum: wss://...
✅ Connected to Ethereum
✅ Wallet configured
📊 Submit intents to build batches
```

## Contract Functions Called

The Rust code calls these functions from your FisherProduction.sol:

### `submitBatchOptimized`
```solidity
function submitBatchOptimized(
    IEVVMCore.Payment[] calldata payments,
    bytes[] calldata signatures
) external returns (bool[] memory results)
```

**Rust calls this at:** `relayer.rs:submit_batch_to_ethereum()`

### `estimateGas`
```solidity
function estimateGas(uint256 batchSize) external view returns (
    uint256 estimatedGas,
    uint256 estimatedSavings
)
```

**Rust calls this at:** `relayer.rs:estimate_batch_gas()`

### `calculateChunkSize`
```solidity
function calculateChunkSize(uint256 batchSize) external pure returns (uint256)
```

**Used for:** Williams compression validation

## Flow

```
1. User submits intent → Rust Fisher
2. Fisher queues intent
3. When batch ready:
   a. φ-Freeman sort (Rust)
   b. Williams chunking (Rust)
   c. submitBatchOptimized() → Your FisherProduction.sol
4. Transaction confirmed
5. Update metrics
```

## Testing

### Test on Sepolia

```bash
# Build
cargo build --release

# Run with your config
./target/release/fisher-relayer --config config.json
```

### Submit Test Intent

```bash
curl -X POST http://localhost:3000/submit-intent \
  -H "Content-Type: application/json" \
  -d '{
    "id": "test_1",
    "from": "0xYourAddress",
    "to": "0xRecipient",
    "amount": "100",
    "priority": false,
    "nonce": 1,
    "signature": "0x..."
  }'
```

## Monitoring

Fisher exposes metrics at `http://localhost:9090/metrics`:

```
fisher_total_batches 42
fisher_total_intents 1337
fisher_avg_savings_percent 87.5
```

## Next Steps

1. ✅ Get contract addresses from your deployment
2. ✅ Create config.json with your values
3. ✅ Test `cargo run --example run_fisher`
4. ✅ Verify it calls your FisherProduction contract
5. ✅ Build WASM and run in Enarx TDX
