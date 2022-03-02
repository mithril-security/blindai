#!/bin/sh
/opt/intel/sgx-aesm-service/aesm/aesm_service --no-daemon &
cd /root
./app
