precommit:
  #!/usr/bin/env bash

  set -x
  set -e

  pushd server_sgx
  cargo fmt
  cargo clippy --target x86_64-fortanix-unknown-sgx -p blindai_server -- --no-deps -Dwarnings 

  pushd runner
  cargo fmt
  cargo clippy 
  popd
  popd

  pushd client 
  poetry run black --check . 
  poetry run mypy --install-types --non-interactive --ignore-missing-imports --follow-imports=skip
  popd

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
  pushd client
  poetry run coverage run -m pytest --ignore=tests/integration_test.py  --
  popd

  pushd server_sgx
  just run --release &
  popd

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
  coverage html --include=blindai/client.py,blindai/utils.py -d coverage_html
  poetry run python -m http.server 8000 --directory coverage_html/
