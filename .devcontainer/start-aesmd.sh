#!/usr/bin/env bash


## Careful: this file is unused at the moment.

set -e

if test $(id -u) -ne 0; then
    echo "Root privilege is required."
    exit 1
fi

# Make Intel driver accessible from default "vscode" user
chown -R vscode /dev/sgx/

# Create user and group if not exist
id -u aesmd &> /dev/null || \
    useradd -r -U -c "User for aesmd" \
    -d /var/opt/aesmd -s /sbin/nologin aesmd

export LD_LIBRARY_PATH=/opt/intel/sgx-aesm-service/aesm
export AESM_PATH=/opt/intel/sgx-aesm-service/aesm

cd /opt/intel/sgx-aesm-service/aesm

/opt/intel/sgx-aesm-service/aesm/linksgx.sh
/bin/mkdir -p /var/run/aesmd/
/bin/chown -R aesmd:aesmd /var/run/aesmd/
/bin/chmod 0755 /var/run/aesmd/
/bin/chown -R aesmd:aesmd /var/opt/aesmd/
/bin/chmod 0750 /var/opt/aesmd/

sudo --preserve-env=AESM_PATH,LD_LIBRARY_PATH -u aesmd -- '/opt/intel/sgx-aesm-service/aesm/aesm_service' --no-daemon