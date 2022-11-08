#!/usr/bin/env just --justfile

default:
  @just --list

# Run on SGX hardware
run-sgx *args:
  cargo run --target=x86_64-fortanix-unknown-sgx {{args}}

# Run in the simulator
run-simu *args:
  #!/usr/bin/env bash
  set -e

  cargo build --target=x86_64-fortanix-unknown-sgx {{args}}

  binpath=`cargo build --target=x86_64-fortanix-unknown-sgx {{args}} --message-format json 2>/dev/null \
    | jq -r 'select(.reason=="compiler-artifact" and .target.kind==["bin"]) | .executable'` 
  
  ftxsgx-elf2sgxs "$binpath" \
    --heap-size 0x2000000 \
    --ssaframesize 1 \
    --stack-size 0x20000 \
    --threads 10

  ftxsgx-simulator "$binpath.sgxs"

# Execute with valgrind instrumentation
valgrind *args:
  #!/usr/bin/env bash
  set -e

  cargo build --target=x86_64-fortanix-unknown-sgx {{args}}

  binpath=`cargo build --target=x86_64-fortanix-unknown-sgx {{args}} --message-format json 2>/dev/null \
    | jq -r 'select(.reason=="compiler-artifact" and .target.kind==["bin"]) | .executable'`

  ftxsgx-elf2sgxs "$binpath" \
    --heap-size 0x2000000 \
    --ssaframesize 1 \
    --stack-size 0x20000 \
    --threads 10

  valgrind --sigill-diagnostics=no --leak-check=no ftxsgx-simulator "$binpath.sgxs" 

