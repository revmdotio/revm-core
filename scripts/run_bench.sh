#!/bin/bash
set -euo pipefail

echo "=== REVM Core Benchmarks ==="
echo ""

cargo bench --bench routing_bench 2>&1 | tee bench_output.txt

echo ""
echo "=== Benchmark complete ==="
echo "Full results saved to target/criterion/"
