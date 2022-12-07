#/bin/bash -l

set -e

echo cargo fmt
cargo fmt --check
echo cargo clippy
cargo clippy --no-deps -- -Dwarnings
