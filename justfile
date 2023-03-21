#!/usr/bin/env just --justfile

default:
  @just --list


run_pccs:
  #!/usr/bin/env bash
  set -e
  cd /opt/intel/sgx-dcap-pccs
  sudo sed -i '/ApiKey/c\   \"ApiKey\" : \"'$1'\",' default.json 
  sudo npm start pm2 


# Run on SGX hardware
run *args:
  #!/usr/bin/env bash
  set -e

  cargo build --target x86_64-fortanix-unknown-sgx {{args}}

  binpath=`cargo build --target x86_64-fortanix-unknown-sgx {{args}} --message-format json 2>/dev/null \
    | jq -r 'select(.reason=="compiler-artifact" and .target.kind==["bin"]) | .executable'` 

  ftxsgx-elf2sgxs "$binpath" \
    --heap-size 0x4FBA00000 \
    --ssaframesize 1 \
    --stack-size 0x20000 \
    --threads 20

  just generate-manifest-dev "$binpath.sgxs" 
  cp manifest.dev.toml client/blindai_preview/manifest.toml

  just generate-manifest-prod "$binpath.sgxs" 

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
    --heap-size 0x2FBA00000 \
    --ssaframesize 1 \
    --stack-size 0x20000 \
    --threads 20

  just generate-manifest-dev "$binpath.sgxs" 

  just generate-manifest-prod "$binpath.sgxs" 

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

  just generate-manifest-dev "$binpath.sgxs" 

  just generate-manifest-prod "$binpath.sgxs" 

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

  just generate-manifest-dev "$binpath.sgxs" 

  just generate-manifest-prod "$binpath.sgxs" 

  valgrind --sigill-diagnostics=no --leak-check=no ftxsgx-simulator "$binpath.sgxs" 


# generate a manifest.toml for dev purposes, expects path to the sgxs file 
generate-manifest-dev input_sgxs:
  #!/usr/bin/env bash
  set -e
  export mr_enclave=`sgxs-hash {{input_sgxs}}`
  envsubst < manifest.dev.template.toml > manifest.dev.toml

# generate a manifest.toml for prod purposes expects path to the sgxs file
generate-manifest-prod input_sgxs:
  #!/usr/bin/env bash
  set -e
  export mr_enclave=`sgxs-hash {{input_sgxs}}`
  envsubst < manifest.prod.template.toml > manifest.prod.toml
  
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
  poetry run pytest --ignore=integration_test.py

# Run all tests and display combined coverage (don't forget to generate the onnx and npz files before)
test:
  #!/usr/bin/env bash
  set -e
  set -x
  cd client
  poetry run coverage run -m pytest --ignore=tests/integration_test.py  --
  just run --release &

  # We use the helper function `test_ports` because the server might take long to start
  # and we will not know when it is ready to accept connections.

  test_ports(){
    # Test if the ports (9923, 9924) are open
    lsof -i:{9923,9924} | awk -F':' '{print $2}' | awk '{print $1}'

    # Test if the operation was successful
    if [ $? -eq 0 ]; then
      echo 1
    else
      echo 0
    fi
  }
  while [ $(test_ports) -eq 1 ]; do
    echo "Waiting for ports to be opened..."
    sleep 10
  done
  for d in ../tests/*/ ; do
  	if [[ "$d" == *"mobilenet"* ]]; then
      continue
    fi
    onnx_files=($d*.onnx)
    npz_files=($d*.npz)
    poetry run coverage run --append ../tests/assert_correctness.py "${onnx_files[0]}" "${npz_files[0]}"
  done
  killall runner
  coverage html --include=blindai_preview/client.py,blindai_preview/utils.py -d coverage_html
  poetry run python -m http.server 8000 --directory coverage_html/


precommit:
  #!/usr/bin/env bash

  set -x
  set -e

  cargo fmt
  cargo clippy --target x86_64-fortanix-unknown-sgx -p blindai_server -- --no-deps -Dwarnings 

  pushd runner
  cargo fmt
  cargo clippy 
  popd

  pushd client 
  poetry run black --check . 
  poetry run mypy --install-types --non-interactive --ignore-missing-imports --follow-imports=skip
  popd

release:
  #!/usr/bin/env bash
  set -e 
  set -x

  just build --release
  cp manifest.prod.toml client/blindai_preview/manifest.toml

  openssl genrsa -3 3072 > my_key.pem
  sgxs-sign --key my_key.pem  target/x86_64-fortanix-unknown-sgx/release/blindai_server.sgxs   target/x86_64-fortanix-unknown-sgx/release/blindai_server.sig   --xfrm 7/0 --isvprodid 0 --isvsvn 0
  ./runner/target/release/runner target/x86_64-fortanix-unknown-sgx/release/blindai_server.sgxs