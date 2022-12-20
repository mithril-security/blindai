#!/bin/sh
# /opt/intel/sgx-aesm-service/aesm/aesm_service --no-daemon &
cd /opt/intel/sgx-dcap-pccs
sed -i '/ApiKey/c\   \"ApiKey\" : \"'$1'\",' default.json 
npm start pm2 & 

