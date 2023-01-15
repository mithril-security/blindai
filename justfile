#!/usr/bin/env just --justfile

default:
  @just --list


# Run on SGX hardware
run *args:
  #!/usr/bin/env bash
  set -e

  cargo build --target x86_64-fortanix-unknown-sgx {{args}}

  binpath=`cargo build --target x86_64-fortanix-unknown-sgx {{args}} --message-format json 2>/dev/null \
    | jq -r 'select(.reason=="compiler-artifact" and .target.kind==["bin"]) | .executable'` 

  ftxsgx-elf2sgxs "$binpath" \
    --heap-size 0xFBA00000 \
    --ssaframesize 1 \
    --stack-size 0x20000 \
    --threads 20

  just generate-policy-dev "$binpath.sgxs" 

  just generate-policy-prod "$binpath.sgxs" 

  ( cd runner && cargo build --release )

  # ftxsgx-runner "$binpath.sgxs" 

  # Modify the normal runner to the new 
  ./runner/target/release/runner "$binpath.sgxs"
  

# Build for SGX target
build *args:
  #!/usr/bin/env bash
  set -e
  cargo build --target x86_64-fortanix-unknown-sgx {{args}}

  binpath=`cargo build --target x86_64-fortanix-unknown-sgx {{args}} --message-format json 2>/dev/null \
    | jq -r 'select(.reason=="compiler-artifact" and .target.kind==["bin"]) | .executable'` 
    
  ftxsgx-elf2sgxs "$binpath" \
    --heap-size 0x2000000 \
    --ssaframesize 1 \
    --stack-size 0x20000 \
    --threads 20

  just generate-policy-dev "$binpath.sgxs" 

  just generate-policy-prod "$binpath.sgxs" 

  ( cd runner && cargo build --release )

# Check for SGX target
check *args:
  cargo check --target x86_64-fortanix-unknown-sgx {{args}}

# Build for a Linux target (no SGX)
build-no-sgx *args:
  cargo build {{args}}

# Run on a Linux target (no SGX)
run-no-sgx *args:
  cargo run {{args}}

# Run in the simulator
run-simu *args:
  #!/usr/bin/env bash
  set -e

  cargo build --target x86_64-fortanix-unknown-sgx {{args}}

  binpath=`cargo build --target x86_64-fortanix-unknown-sgx {{args}} --message-format json 2>/dev/null \
    | jq -r 'select(.reason=="compiler-artifact" and .target.kind==["bin"]) | .executable'` 
  
  ftxsgx-elf2sgxs "$binpath" \
    --heap-size 0xFBA00000 \
    --ssaframesize 1 \
    --stack-size 0x40000 \
    --threads 20

  just generate-policy-dev "$binpath.sgxs" 

  just generate-policy-prod "$binpath.sgxs" 

  ftxsgx-simulator "$binpath.sgxs"

# Execute with valgrind instrumentation
valgrind *args:
  #!/usr/bin/env bash
  set -e

  cargo build --target x86_64-fortanix-unknown-sgx {{args}}

  binpath=`cargo build --target x86_64-fortanix-unknown-sgx {{args}} --message-format json 2>/dev/null \
    | jq -r 'select(.reason=="compiler-artifact" and .target.kind==["bin"]) | .executable'`

  ftxsgx-elf2sgxs "$binpath" \
    --heap-size 0x2000000 \
    --ssaframesize 1 \
    --stack-size 0x20000 \
    --threads 20

  just generate-policy-dev "$binpath.sgxs" 

  just generate-policy-prod "$binpath.sgxs" 

  valgrind --sigill-diagnostics=no --leak-check=no ftxsgx-simulator "$binpath.sgxs" 


# generate a policy.toml for dev purposes, expects path to the sgxs file 
generate-policy-dev input_sgxs:
  #!/usr/bin/env bash
  export mr_enclave=`sgxs-hash {{input_sgxs}}`
  envsubst < policy.dev.template.toml > policy.dev.toml

# generate a policy.toml for prod purposes expects path to the sgxs file
generate-policy-prod input_sgxs:
  #!/usr/bin/env bash
  export mr_enclave=`sgxs-hash {{input_sgxs}}`
  envsubst < policy.prod.template.toml > policy.prod.toml
  
# Build and serve locally the documentation
doc:
  #!/usr/bin/env bash
  cd client \
  && poetry install --no-root \
  && poetry run pip install -r ../docs/requirements.txt \
  && poetry run bash -c 'cd .. && . docs/generate_api_reference.sh' \
  && poetry run mkdocs serve -f ../mkdocs.yml

basic_test:
  #!/usr/bin/env bash
  set -e 
  set -x
  cd client/tests
  poetry run pytest

# Run all tests and display combined coverage (don't forget to generate the onnx and npz files before)
test:
  #!/usr/bin/env bash
  set -e 
  cd client
  poetry run coverage run -m pytest
  just run &
  sleep 15
  for d in ../tests/*/ ; do
    onnx_files=($d*.onnx)
    npz_files=($d*.npz)
    poetry run coverage run --append ../tests/assert_correctness.py "${onnx_files[0]}" "${npz_files[0]}"
  done
  killall runner
  coverage html --include=client/client.py,client/utils.py -d coverage_html
  poetry run python -m http.server 8000 --directory coverage_html/