#!/bin/sh

set -x
set -e

/nitriding -fqdn example.com  -extport 8443  -intport 8080 &
echo "[sh] Started nitriding."
sleep 1

mkdir -p /run/sshd
/usr/sbin/sshd -D -e