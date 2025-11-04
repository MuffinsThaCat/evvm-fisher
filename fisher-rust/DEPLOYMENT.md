# 🚀 Fisher Relayer Deployment Guide

## Production Deployment Options

### Option 1: Native Binary (Fastest)

#### Prerequisites
- Rust 1.75+
- Linux x86_64 or ARM64

#### Steps

```bash
# Build release binary
cargo build --release

# Copy binary
sudo cp target/release/fisher-relayer /usr/local/bin/

# Create user and directories
sudo useradd -r -s /bin/false fisher
sudo mkdir -p /etc/fisher /var/lib/fisher
sudo chown fisher:fisher /var/lib/fisher

# Copy configuration
sudo cp config.json /etc/fisher/config.json
sudo chown root:fisher /etc/fisher/config.json
sudo chmod 640 /etc/fisher/config.json

# Install systemd service
sudo cp fisher-relayer.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable fisher-relayer
sudo systemctl start fisher-relayer

# Check status
sudo systemctl status fisher-relayer
```

---

### Option 2: Docker (Portable)

#### Prerequisites
- Docker 20.10+
- Docker Compose 2.0+

#### Steps

```bash
# Edit configuration
cp config.example.json config.json
# Edit config.json with your settings

# Build and run
docker-compose up -d

# Check logs
docker-compose logs -f fisher-relayer

# Check health
docker-compose ps
```

---

### Option 3: Kubernetes (Scalable)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: fisher-relayer
spec:
  replicas: 3
  selector:
    matchLabels:
      app: fisher-relayer
  template:
    metadata:
      labels:
        app: fisher-relayer
    spec:
      containers:
      - name: fisher-relayer
        image: fisher-relayer:1.0.0
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        volumeMounts:
        - name: config
          mountPath: /data/config.json
          subPath: config.json
        livenessProbe:
          exec:
            command:
            - /usr/local/bin/fisher-relayer
            - --health-check
          initialDelaySeconds: 5
          periodSeconds: 30
        resources:
          requests:
            memory: "256Mi"
            cpu: "500m"
          limits:
            memory: "512Mi"
            cpu: "1000m"
      volumes:
      - name: config
        configMap:
          name: fisher-config
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: fisher-config
data:
  config.json: |
    {
      "rpc_url": "wss://eth-sepolia.g.alchemy.com/v2/YOUR_KEY",
      "fisher_address": "0x...",
      "evvm_core_address": "0x...",
      "min_batch_size": 10,
      "max_batch_size": 1000,
      "batch_interval_ms": 5000,
      "enable_attestation": true
    }
```

---

### Option 4: Enarx TEE (Most Secure)

#### Prerequisites
- Enarx 0.7+
- Intel TDX or AMD SEV-SNP hardware

#### Steps

```bash
# Build for WASM
cargo build --target wasm32-wasi --release

# Create Enarx.toml
cat > Enarx.toml <<EOF
[[files]]
kind = "stdin"

[[files]]
kind = "stdout"

[[files]]
kind = "stderr"

[[files]]
name = "/config.json"
kind = "inline"
data = """
{
  "rpc_url": "wss://eth-sepolia.g.alchemy.com/v2/YOUR_KEY",
  "fisher_address": "0x...",
  "evvm_core_address": "0x...",
  "enable_attestation": true
}
"""
EOF

# Run in TEE
enarx run \
  --backend tdx \
  --wasmcfgfile Enarx.toml \
  target/wasm32-wasi/release/fisher-relayer.wasm
```

---

## Configuration

### Minimal config.json

```json
{
  "rpc_url": "wss://eth-sepolia.g.alchemy.com/v2/YOUR_ALCHEMY_KEY",
  "fisher_address": "0x8F111895ddAD9e672aD2BCcA111c46E1eADA5E90",
  "evvm_core_address": "0xF817e9ad82B4a19F00dA7A248D9e556Ba96e6366",
  "min_batch_size": 10,
  "max_batch_size": 1000,
  "batch_interval_ms": 5000,
  "enable_attestation": false,
  "private_key": "0xYourPrivateKeyHere"
}
```

### Production config.json

```json
{
  "rpc_url": "wss://eth-mainnet.g.alchemy.com/v2/YOUR_ALCHEMY_KEY",
  "fisher_address": "0xYourProductionFisherAddress",
  "evvm_core_address": "0xYourProductionEVVMCoreAddress",
  "min_batch_size": 50,
  "max_batch_size": 5000,
  "batch_interval_ms": 3000,
  "enable_attestation": true,
  "private_key": "0xYourProductionPrivateKey"
}
```

---

## Monitoring

### Health Check

```bash
# Check if running
fisher-relayer --health-check

# Docker
docker exec fisher-relayer fisher-relayer --health-check
```

### Logs

```bash
# Systemd
sudo journalctl -u fisher-relayer -f

# Docker
docker logs -f fisher-relayer

# Kubernetes
kubectl logs -f deployment/fisher-relayer
```

### Metrics

Fisher exports Prometheus metrics on `/metrics`:

```
fisher_total_batches        # Total batches processed
fisher_total_intents        # Total intents processed
fisher_avg_savings_percent  # Average gas savings
fisher_avg_batch_size       # Average batch size
```

Example Prometheus config:

```yaml
scrape_configs:
  - job_name: 'fisher'
    static_configs:
      - targets: ['localhost:8080']
```

---

## Security Best Practices

### 1. Private Key Management

**Never** commit private keys to version control.

Use environment variables or secret managers:

```bash
# Using environment variable
export FISHER_PRIVATE_KEY="0x..."
fisher-relayer --config config.json

# Using Kubernetes secrets
kubectl create secret generic fisher-secrets \
  --from-literal=private-key=0x...
```

### 2. TLS/HTTPS

Always use WSS (WebSocket Secure) for RPC:

```json
{
  "rpc_url": "wss://..." 
}
```

### 3. Firewall

Only expose necessary ports:

```bash
# Allow only from specific IPs
sudo ufw allow from 192.168.1.0/24 to any port 8080
```

### 4. TEE Attestation

Enable for production:

```json
{
  "enable_attestation": true
}
```

---

## Troubleshooting

### Binary won't start

```bash
# Check configuration
fisher-relayer --health-check

# Check permissions
ls -l /usr/local/bin/fisher-relayer
# Should be executable: -rwxr-xr-x
```

### Connection failures

```bash
# Test RPC connectivity
curl -X POST \
  -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  YOUR_RPC_URL
```

### Out of memory

Increase resource limits:

```ini
# /etc/systemd/system/fisher-relayer.service
[Service]
MemoryMax=1G
```

### High gas usage

Check batch configuration:

```json
{
  "min_batch_size": 50,    // Increase for better savings
  "max_batch_size": 5000   // Higher = more efficient
}
```

---

## Performance Tuning

### For High Throughput

```json
{
  "min_batch_size": 100,
  "max_batch_size": 10000,
  "batch_interval_ms": 1000
}
```

### For Low Latency

```json
{
  "min_batch_size": 10,
  "max_batch_size": 100,
  "batch_interval_ms": 500
}
```

### For Maximum Savings

```json
{
  "min_batch_size": 500,
  "max_batch_size": 10000,
  "batch_interval_ms": 10000
}
```

---

## Upgrading

```bash
# Native
cd fisher-rust
git pull
cargo build --release
sudo systemctl stop fisher-relayer
sudo cp target/release/fisher-relayer /usr/local/bin/
sudo systemctl start fisher-relayer

# Docker
docker-compose pull
docker-compose up -d

# Kubernetes
kubectl set image deployment/fisher-relayer \
  fisher-relayer=fisher-relayer:1.1.0
```

---

## Support

- **Documentation**: `README.md`, `COMPLETE_SYSTEM.md`
- **Examples**: `examples/run_fisher.rs`
- **Tests**: `cargo test`
- **Health Check**: `fisher-relayer --health-check`

---

**You're ready for production! 🚀**
