# File Reporting Benchmarks

This directory contains comprehensive benchmarks for the imixv2 file reporting system, implementing three unique architectural patterns to measure performance across the full chain: disk I/O → encryption → transport.

## Benchmark Patterns

### Pattern 1: Component-Level Micro-Benchmarks (`component_benchmarks.rs`)

**Philosophy**: Decompose the file reporting pipeline into independent stages and benchmark each in isolation to identify specific bottlenecks.

**What it measures**:
- **Stage 1 - Disk I/O**: `std::fs::read()` vs chunked reading (1KB, 256KB, 2MB)
- **Stage 2 - Encryption**: XChaCha20-Poly1305 throughput and overhead
- **Stage 3 - Serialization**: Protobuf encoding performance

**When to use**: During optimization to pinpoint exact bottlenecks.

**Run**:
```bash
cargo bench --package eldritch-libreport --bench component_benchmarks
```

### Pattern 2: End-to-End Integration Benchmarks (`end_to_end.rs`)

**Philosophy**: Benchmark the complete file reporting flow through the actual `Agent` trait implementation using realistic network simulation.

**What it measures**:
- End-to-end latency with different network conditions (fast/typical/slow)
- Effective throughput (MB/s)
- Network efficiency (encrypted vs raw bytes overhead)
- Real-world performance with actual encryption and network latency

**When to use**: For production validation and acceptance testing.

**Run**:
```bash
cargo bench --package eldritch-libreport --bench end_to_end
```

### Pattern 3: Comparative A/B Benchmarks (`comparative.rs`)

**Philosophy**: Implement multiple file reporting strategies side-by-side and compare them directly to quantify performance differences.

**Strategies compared**:
1. **v2_current**: `std::fs::read()` + single chunk (current implementation)
2. **v1_legacy**: 1KB streaming chunks (old v1 approach)
3. **streaming_2mb**: 2MB streaming chunks (modern approach)
4. **adaptive**: Dynamic chunk size based on file size

**What it measures**:
- Latency comparison across strategies
- Memory overhead (peak_memory/file_size)
- Throughput degradation as file size increases
- Encryption overhead per strategy
- Crossover points where streaming becomes superior

**When to use**: For architectural decision-making and validating regressions.

**Run**:
```bash
cargo bench --package eldritch-libreport --bench comparative
```

## Running Benchmarks

### Run All Benchmarks
```bash
cargo bench --package eldritch-libreport
```

### Run Specific Pattern
```bash
cargo bench --package eldritch-libreport --bench comparative
cargo bench --package eldritch-libreport --bench component_benchmarks
cargo bench --package eldritch-libreport --bench end_to_end
```

### Run Specific Benchmark by Name
```bash
# Run only 1MB file benchmarks
cargo bench --package eldritch-libreport --bench comparative -- "1MB"

# Run only disk read benchmarks
cargo bench --package eldritch-libreport --bench component_benchmarks -- "stage1_disk_read"

# Run only network efficiency benchmarks
cargo bench --package eldritch-libreport --bench end_to_end -- "network_efficiency"
```

### Test Mode (Quick Validation)
```bash
# Run benchmarks in test mode (single iteration)
cargo bench --package eldritch-libreport --bench comparative -- --test
```

## File Sizes Tested

All benchmarks test the following file sizes (per user requirements):
- **1 KB** - Small config files
- **1 MB** - Typical logs and documents
- **5 MB** - Large logs
- **10 MB** - Large files and binaries

Pattern 3 (Comparative) also includes:
- **256 KB** - Medium files
- **2 MB** - Chunk size boundary testing

## Network Conditions (End-to-End Benchmarks)

- **Fast**: 0.1ms latency, 1 GB/s bandwidth (local network)
- **Typical**: 5ms latency, 10 MB/s bandwidth (typical network)
- **Slow**: 20ms latency, 1 MB/s bandwidth (slow network)

## Metrics Measured

### Component-Level
- Latency per stage (disk, encryption, serialization)
- Throughput (MB/s) per component
- Memory usage per stage
- Size overhead (encryption: 72 bytes, protobuf: varies)

### End-to-End
- Total end-to-end latency
- Effective throughput (MB/s)
- Network efficiency (overhead ratio)
- Peak memory during operation

### Comparative
- Latency distribution (p50, p95, p99) per strategy
- Memory efficiency ratio
- Throughput comparison across file sizes
- Chunk count analysis
- Crossover analysis (where streaming wins)

## Benchmark Output

Criterion generates detailed reports in `target/criterion/`:
- HTML reports with graphs
- Statistical analysis (mean, std dev, outliers)
- Comparison with baseline (if available)

View the HTML reports:
```bash
open target/criterion/report/index.html
```

## Architecture

```
benches/
├── comparative.rs              # Pattern 3: A/B comparison
├── component_benchmarks.rs     # Pattern 1: Component-level
├── end_to_end.rs              # Pattern 2: End-to-end integration
├── fixtures/
│   └── mod.rs                 # Shared test file generation
├── strategies/
│   └── mod.rs                 # File reporting strategies (v2, v1, streaming, adaptive)
├── stages/
│   ├── disk_read.rs           # Disk I/O benchmarks
│   ├── encryption.rs          # Encryption benchmarks
│   └── serialization.rs       # Protobuf benchmarks
└── mocks/
    └── realistic_agent.rs     # Realistic network simulation
```

## Key Findings (To Be Updated After Running)

After running benchmarks, update this section with key findings:

### Memory Efficiency
- v2 current: Uses O(file_size) memory
- Streaming: Uses constant O(chunk_size) memory
- Expected: Streaming should use ~2MB regardless of file size

### Latency
- Small files (<256KB): v2 may be faster (less overhead)
- Large files (>5MB): Streaming should win (constant memory)

### Encryption Overhead
- v2: 72 bytes total (single encryption)
- Streaming: 72 bytes × num_chunks
- For 10MB file with 2MB chunks: 5 × 72 = 360 bytes overhead

### Throughput
- Expected to degrade as file size increases for v2 (memory allocation)
- Expected to remain constant for streaming

## Troubleshooting

### Benchmarks Take Too Long
```bash
# Run in test mode for quick validation
cargo bench --package eldritch-libreport -- --test

# Or limit to specific benchmarks
cargo bench --package eldritch-libreport --bench comparative -- "1KB"
```

### Out of Memory Errors
Reduce the maximum file size tested or increase available memory.

### Inconsistent Results
- Close other applications
- Disable CPU frequency scaling
- Run multiple times and compare
- Use `nice` to increase priority: `nice -n -20 cargo bench`

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [File Reporting Plan](/home/vscode/.claude/plans/mossy-snuggling-rabin.md)
- [eldritch-libreport README](../README.md)
