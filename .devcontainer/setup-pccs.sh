#!/bin/sh

# PCCS install configuration variables
INSTALL='Y'
HTTP_PROXY=''
HTTPS_PROXY=''
CONFIGURE='Y'
# default port is 8081, any change should be reflected in /etc/sgx_default_qcnl.conf
HTTPS_PORT='8081'
LOCAL_ONLY='Y'
# default is LAZY
CACHING='' 
# left empty as this value is specified when doing docker run
API_KEY=''

# command used to generate password date +%s | sha256sum | base64 | head -c 32 ; echo
ADMIN_PSW='NGQyMWIzND'
USER_PSW='ZThhZjM3ZT'

#insecure certificate specifications
GENERATE_CERT='Y'

COUNTRY_CODE='FR'
STATE='ile-de-france'
CITY='paris'
ORGANIZATION='mithril-security'
UNIT='development'
COMMON_NAME='MS'
EMAIL=''

# extra specifications
CHALLENGE_PSW=''
OPTIONAL_COMPANY_NAME=''

cd /opt/intel/sgx-dcap-pccs

echo $INSTALL'\n'$HTTP_PROXY'\n'$HTTPs_PROXY'\n'$CONFIGURE'\n' \
        $HTTPS_PORT'\n'$LOCAL_ONLY '\n'$API_KEY'\n'$CACHING'\n' \
        $ADMIN_PSW'\n'$ADMIN_PSW'\n'$USER_PSW'\n'$USER_PSW'\n' \
        $GENERATE_CERT'\n'$COUNTRY_CODE'\n'$STATE'\n'$CITY'\n' \
        $ORGANIZATION'\n'$UNIT'\n'$COMMON_NAME'\n'$EMAIL'\n' \
        $CHALLENGE_PSW'\n'$OPTIONAL_COMPANY_NAME'\n' | ./install.sh