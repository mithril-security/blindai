#!/bin/bash

set -e
set -x

TEMPD=$(mktemp -d)

# Exit if the temp directory wasn't created successfully.
if [ ! -e "$TEMPD" ]; then
    >&2 echo "Failed to create temp directory"
    exit 1
fi

# Make sure the temp directory gets removed on script exit.
trap "exit 1"           HUP INT PIPE QUIT TERM
trap 'rm -rf "$TEMPD"'  EXIT

pushd $TEMPD

sudo apt-get update
sudo apt-get install -y udev

git clone https://github.com/mithril-security/aws-nitro-enclaves-cli.git --depth 1
cd aws-nitro-enclaves-cli
make nitro-cli
make vsock-proxy
sudo make NITRO_CLI_INSTALL_DIR=/ install
source /etc/profile.d/nitro-cli-env.sh
nitro-cli-config -i -s
# We shouldn't have to change the owner of these dev/directory
# but we've got problem if we don't do that so for now, it does the job :
sudo chown vscode /dev/nitro_enclaves
sudo chown vscode /run/nitro_enclaves

popd

# Generate an ECDSA SSH key (with no password)
ssh-keygen -t ecdsa -q -f $HOME/.ssh/id_ecdsa -N ''

git clone https://github.com/containers/gvisor-tap-vsock.git
cd gvisor-tap-vsock/
make