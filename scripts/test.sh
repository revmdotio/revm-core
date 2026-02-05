#!/bin/bash
set -euo pipefail

echo "=== REVM Core Test Suite ==="
echo ""

echo "[1/3] Running unit tests..."
cargo test --lib 2>&1

echo ""
echo "[2/3] Running integration tests..."
cargo test --test integration_test 2>&1

echo ""
echo "[3/3] Running doc tests..."
cargo test --doc 2>&1

echo ""
echo "=== All tests passed ==="
