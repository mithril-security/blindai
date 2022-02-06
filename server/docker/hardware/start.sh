#!/bin/sh
/opt/intel/sgx-aesm-service/aesm/aesm_service --no-daemon &
cd /opt/intel/sgx-dcap-pccs
sed -i '/ApiKey/c\   \"ApiKey\" : \"'$1'\",' config/default.json 
npm start pm2 & 
cd /root/
./app