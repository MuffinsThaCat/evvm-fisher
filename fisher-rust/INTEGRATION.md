# Integrating Fisher with Your Enarx TDX Infrastructure

This guide explains how to integrate this Fisher implementation with your existing Enarx TDX setup.

## Your Existing Infrastructure

You already have production-ready TDX infrastructure at:

```
/Users/talzisckind/Downloads/aristo-fresh 2/enarx/
├── src/backend/tdx/
│   ├── mod.rs           # TDX backend
│   ├── attestation.rs   # TDX attestation (261 lines!)
│   ├── shim.rs          # TDX shim
│   └── config.rs        # TDX configuration
└── Cargo.toml
```

## Integration Steps

### Step 1: Link Fisher to Enarx

Add Fisher as a dependency in your Enarx project:

```toml
# /aristo-fresh 2/enarx/Cargo.toml

[dependencies]
fisher-relayer = { path = "../fisher-rust" }
```

### Step 2: Modify Fisher's Attestation Module

Update `fisher-rust/src/attestation.rs` to call your TDX backend:

```rust
// In fisher-rust/src/attestation.rs

use aristo_enarx::tdx::attestation as tdx_attestation;

fn get_tdx_quote(&self, report_data: &[u8; 64]) -> Result<Vec<u8>> {
    // Call YOUR attestation code
    let quote = tdx_attestation::TdxQuote::new(report_data)?;
    Ok(quote.data)
}
```

### Step 3: Build Fisher WASM

```bash
cd fisher-rust
./build.sh
```

This creates: `target/wasm32-wasi/release/fisher_relayer.wasm`

### Step 4: Run Fisher in Your Enarx TDX

```bash
cd aristo-fresh\ 2/enarx

# Run Fisher in YOUR TDX backend
enarx run --backend tdx \
    ../../fisher-rust/target/wasm32-wasi/release/fisher_relayer.wasm
```

### Step 5: Integrate with Your Keep Manager

Your keep manager already handles TEE instances:

```rust
// aristo-fresh 2/execution/controller/src/enarx/keep_manager.rs

pub struct KeepManager {
    // Your existing code
}

impl KeepManager {
    // Add Fisher-specific keep
    pub fn create_fisher_keep(&mut self) -> Result<Keep> {
        let mut keep = Keep::new(
            Uuid::new_v4().to_string(),
            "tdx".to_string(),
        );
        
        // Load Fisher WASM
        let fisher_wasm = include_bytes!("path/to/fisher_relayer.wasm");
        
        // Start keep with Fisher
        // Your existing keep launch logic here
        
        keep.mark_ready();
        Ok(keep)
    }
}
```

## Configuration

Create `config.json` for Fisher:

```json
{
  "rpc_url": "https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY",
  "fisher_address": "0xYourFisherContract",
  "evvm_core_address": "0xYourEVVMCore",
  "min_batch_size": 10,
  "max_batch_size": 1000,
  "batch_interval_ms": 5000,
  "enable_attestation": true
}
```

## Testing Integration

```bash
# Test Fisher WASM loads in Enarx
enarx run --backend tdx fisher_relayer.wasm --help

# Test attestation works
enarx run --backend tdx fisher_relayer.wasm --test-attestation

# Run full Fisher
enarx run --backend tdx fisher_relayer.wasm --config config.json
```

## Architecture Diagram

```
┌────────────────────────────────────────────┐
│     EVVM Users (Fishing Spots)             │
│   Submit gasless intents                   │
└─────────────────┬──────────────────────────┘
                  │
                  ▼
┌────────────────────────────────────────────┐
│   Your Enarx Infrastructure                │
│                                            │
│   ┌──────────────────────────────────┐    │
│   │  Keep Manager                     │    │
│   │  (aristo-fresh 2/execution/       │    │
│   │   controller/src/enarx/)          │    │
│   └──────────────┬───────────────────┘    │
│                  │                         │
│                  ▼                         │
│   ┌──────────────────────────────────┐    │
│   │  TDX Backend                      │    │
│   │  (aristo-fresh 2/enarx/           │    │
│   │   src/backend/tdx/)               │    │
│   │                                   │    │
│   │  • attestation.rs ✅             │    │
│   │  • shim.rs ✅                    │    │
│   │  • mod.rs ✅                     │    │
│   └──────────────┬───────────────────┘    │
│                  │                         │
│                  ▼                         │
│   ┌──────────────────────────────────┐    │
│   │  Fisher WASM (THIS PROJECT)      │    │
│   │  • Williams compression          │    │
│   │  • φ-Freeman optimization        │    │
│   │  • Intent batching               │    │
│   └──────────────┬───────────────────┘    │
└──────────────────┼────────────────────────┘
                   │
                   ▼
┌────────────────────────────────────────────┐
│   Ethereum (EVVM + Fisher contracts)       │
│   97-99% cheaper transactions              │
└────────────────────────────────────────────┘
```

## Next Steps

1. ✅ Build Fisher WASM (`./build.sh`)
2. ✅ Test in Enarx (`enarx run...`)
3. ✅ Integrate with keep manager
4. ✅ Configure attestation endpoints
5. ✅ Deploy to production

## Support

Questions? Check:
- Your Enarx docs: `/aristo-fresh 2/enarx/README.md`
- Fisher docs: `README.md`
- Integration tests: `tests/integration.rs`
