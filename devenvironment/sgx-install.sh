#!/usr/bin/env bash
#
# Env variable "SGX_INSTALL_AS_ROOT" can be useful
#   when building in a docker image as root.
# Env variable "TEMP_DEPENDENCIES" is to remove the temporary dependencies
#   at the end of the SGX installation.
# Env variable "DISTRO_CODENAME" is to configure the repository for the SGX
#   drivers installation.
# Env variable "DISTRO_NAME" is to configure from which ubuntu distro directory
#   download the SGX SDK (ubuntu18.04-server or ubuntu20.04-server).
# Env variable "SGX_SIM" is to install SGX even if the hardware doesn't support it.
# Env variable "NO_DCAP_DRIVER" is to not install SGX DCAP driver.
#

########## deb based ##########

test_sgx_url=https://raw.githubusercontent.com/ayeks/SGX-hardware/master/test-sgx.c
if [ ! -f "./.env.vars" ] ; then
    SGX_VERSION=2.15.1
    DCAP_VERSION=1.12.1
else
    SGX_VERSION=$(. ./.env.vars && echo $SGX_VERSION)
    DCAP_VERSION=$(. ./.env.vars && echo $DCAP_VERSION)
fi
# SGX DCAP URL
dcap_version_url=https://download.01.org/intel-sgx/sgx-dcap/$DCAP_VERSION/linux/SHA256SUM_dcap_$DCAP_VERSION.cfg

# SGX LINUX URL
objdump_name=as.ld.objdump.r4.tar.gz
sgx_linux_url=https://download.01.org/intel-sgx/sgx-linux/$SGX_VERSION
sgx_version_url=$sgx_linux_url/SHA256SUM_linux_$SGX_VERSION.cfg
sgx_distro_url=$sgx_linux_url/distro
sgx_objdump_url=$sgx_linux_url/$objdump_name

# SGX REPO URL
deb_repo_url=https://download.01.org/intel-sgx/sgx_repo/ubuntu
deb_repo_key=$deb_repo_url/intel-sgx-deb.key
deb_dists_url=$deb_repo_url/dists/

######### dependencies #########

declare -a deb_deps_temp=(
    wget
    gnupg
    curl
    software-properties-common
    build-essential
    make
    git
    dkms
)

declare -a deb_deps_sgx=(
    sgx-aesm-service
    libsgx-ae-epid
    libsgx-ae-le
    libsgx-ae-pce
    libsgx-aesm-ecdsa-plugin
    libsgx-aesm-epid-plugin
    libsgx-aesm-launch-plugin
    libsgx-aesm-pce-plugin
    libsgx-aesm-quote-ex-plugin
    libsgx-enclave-common
    libsgx-epid
    libsgx-launch
    libsgx-quote-ex
    libsgx-uae-service
    libsgx-urts
    libsgx-ae-pce
)

declare -a deb_deps_dcap=(
    libsgx-ae-qe3
    libsgx-pce-logic
    libsgx-qe3-logic
    libsgx-ra-network
    libsgx-ra-uefi
    libsgx-dcap-ql
    libsgx-dcap-quote-verify
    libsgx-dcap-default-qpl
)

########################################

check_sgx()
{
    curl -sSL $test_sgx_url | gcc -o /tmp/test-sgx -xc -
    EXIT_STATUS=$?
    if ! (exit $EXIT_STATUS) ; then
        echo "[⚠️ ] Warning: Could not verify SGX-ready with 'SGX+FLC' support with the test:" >&2
        echo "$test_sgx_url"
        return 1
    fi
    support=$(/tmp/test-sgx | grep 'sgx launch control' | awk '{print $NF}')
    sgx1_support=$(/tmp/test-sgx | grep 'sgx 1 supported' | awk '{print $NF}')
    sgx2_support=$(/tmp/test-sgx | grep 'sgx 2 supported' | awk '{print $NF}')
    rm /tmp/test-sgx
    if [ $support -eq 1 ] ; then
        echo "🌟 You have an SGX-ready device with SGX+FLC support!"
    else
        echo "⛔ You don't have an SGX-ready device with SGX+FLC support" >&2
        return 0
    fi
    if [ $sgx2_support -eq 1 ] ; then
        echo "✅ You have [SGX 2] support, this is the recommended option for it's better performance and memory availability."
    fi
    if [ $sgx1_support -eq 1 ] ; then
        echo "✅ You have [SGX 1] support, which is memory availability is limited to 128mb."
    fi
    return 1
}

unrecognized_distro()
{
    echo "Unrecognized linux version, needs manual installation, check the documentation:" >&2
    echo "https://blindai.mithrilsecurity.io/en/latest/docs/advanced/build-from-sources/server/" >&2
    exit 1
}

verify_deps()
{
    echo "Verifying dependencies..."
    args=("$@")
    checkcmd=("${args[0]}")
    packages=("${args[@]:1}")
    EXIT_STATUS=0
    missing_status=0
    for package in "${packages[@]}"; do
        $checkcmd $package > /dev/null 2>&1
        EXIT_STATUS=$?
        if ! (exit $EXIT_STATUS) ; then
            echo "[⚠️ ]" $package "(missing)" >&2
            missing_status=$EXIT_STATUS
        else
            echo "[✔️ ]" $package
        fi
    done
    return $missing_status
}

missing_deps()
{
    install_fn=$1
    if [ "$(id -u)" -ne 0 ] ; then
        command -v sudo > /dev/null 2>&1
        EXIT_STATUS=$?
        if ! (exit $EXIT_STATUS) ; then
            echo "It seems sudo is not installed and there are dependencies missing, to install them:" >&2
            echo "Run this script without SGX_INSTALL_AS_ROOT set and with superuser privileges" >&2
            echo "Example: su -c ./build.sh && ./build.sh # To install and then build"
            exit $EXIT_STATUS
        else
            sudo $0
        fi
    else
        $install_fn
    fi
    EXIT_STATUS=$?
    if ! (exit $EXIT_STATUS) ; then
        exit $EXIT_STATUS
    fi
}

############## distro deps ##############

get_selection()
{
    local url=$1
    local sname=$2
    local gvar=$3
    local envar=$4

    if [ -z "${!envar}" ] ; then
        options=($(curl -sSL $url | grep -oP  '(?<=HREF=")[^"]+(?=/")' | awk -F'/' '{print $NF}'))
        options+=("$sname not available")
        echo -e "\n💽 ${sname^^}"
        PS3="Please select a compatible option with your system: "
        select option in "${options[@]}"
        do
            case $option in
                "$sname not available")
                    echo "Please install SGX drivers manually"
                    exit 0
                    ;;
                *)
                    if [ ! -z "${option}" ] ; then
                        echo "$sname selected: $option"
                        declare -g $gvar=$option
                        break
                    else
                        echo "⚠️  $sname unrecognized" >&2
                    fi
                    ;;
            esac
        done
    else
        declare -g $gvar=${!envar}
        echo "$sname selected: ${!envar}"
    fi
}

install_deb_sgx_psw()
{
    get_selection $deb_dists_url "codename" distro_codename "DISTRO_CODENAME"
    # Config repository
    if [ ! -f "/etc/apt/sources.list.d/intel-sgx.list" ] ; then
        echo "deb $deb_repo_url $distro_codename main" | tee -a /etc/apt/sources.list.d/intel-sgx.list
        curl -fsSL $deb_repo_key | apt-key add -
        apt-get update
    fi

    # Add SGX and DCAP versions to packages
    declare -g sgx_version=$(curl -sSL $sgx_version_url | grep -oP "\d+\.\d+\.\d+\.\d+" | tail -1)
    declare -g dcap_version=$(curl -sSL $dcap_version_url | grep -oP "\d+\.\d+\.\d+\.\d+" | tail -1)

    echo "ℹ️  Versions: SGX $sgx_version - DCAP $dcap_version"

    # SGX dependencies
    for ver in "${!deb_deps_sgx[@]}"
    do
        sgx_suffix=$(apt-cache policy "${deb_deps_sgx[$ver]}" | grep "$sgx_version.*$distro_codename" | awk '{print $1}')
        if [[ "${sgx_suffix}" == *"$sgx_version"* ]] ; then
            deb_deps_sgx[$ver]="${deb_deps_sgx[$ver]}=${sgx_suffix}"
            echo "- ${deb_deps_sgx[$ver]}"
        elif [[ "${sgx_suffix}" == *"Installed"* ]] ; then
            echo "- ${deb_deps_sgx[$ver]} Installed"
            deb_deps_sgx[$ver]=""
        else
            echo "[⚠️ ] Error: Could not find $sgx_version-${distro_codename}1 version" >&2
            exit 1
        fi
    done
    # DCAP dependencies
    for ver in "${!deb_deps_dcap[@]}"
    do
        dcap_suffix=$(apt-cache policy "${deb_deps_dcap[$ver]}" | grep "$dcap_version.*$distro_codename" | awk '{print $1}')
        if [[ "${dcap_suffix}" == *"$dcap_version"* ]] ; then
            deb_deps_dcap[$ver]="${deb_deps_dcap[$ver]}=${dcap_suffix}"
            echo "- ${deb_deps_dcap[$ver]}"
        elif [[ "${dcap_suffix}" == *"Installed"* ]] ; then
            echo "- ${deb_deps_dcap[$ver]} Installed"
            deb_deps_dcap[$ver]=""
        else
            echo "[⚠️ ] Error: Could not find $dcap_version-${distro_codename}1 version" >&2
            exit 1
        fi
    done

    # Install SGX PSW
    echo "Installing ${deb_deps_sgx[@]} ${deb_deps_dcap[@]}"
    apt-get -y --allow-downgrades install "${deb_deps_sgx[@]}" "${deb_deps_dcap[@]}"
    EXIT_STATUS=$?
    return $EXIT_STATUS
}

install_sgx_sdk()
{
    get_selection $sgx_distro_url "distro" distro_name "DISTRO_NAME"
    base_bin_url=$sgx_distro_url/$distro_name
    if [ ! -d "/opt/sgxsdk/" ] ; then
        # Intel's Architectural Enclave Service Manager
        mkdir -p /var/run/aesmd
        ln -s /usr/lib/x86_64-linux-gnu/libdcap_quoteprov.so.1 /usr/lib/x86_64-linux-gnu/libdcap_quoteprov.so

        # Install the SGX SDK
        sdkbin=sgx_linux_x64_sdk_$sgx_version.bin
        sdkbin_url=$sgx_distro_url/$distro_name/$sdkbin
        echo "Downloading $sdkbin_url..."
        wget $sdkbin_url
        chmod u+x $sdkbin
        echo -e 'no\n/opt' | ./$sdkbin
        rm $sdkbin
        export SGX_SDK=/opt/sgxsdk
        export PATH=$PATH:$SGX_SDK/bin:$SGX_SDK/bin/x64
        export PKG_CONFIG_PATH=$PKG_CONFIG_PATH:$SGX_SDK/pkgconfig
        if [ -z "${LD_LIBRARY_PATH}" ] ; then
            export LD_LIBRARY_PATH=/usr/lib:/usr/local/lib:/opt/sgxsdk/sdk_libs:/opt/intel/sgx-aesm-service/aesm
        else
            export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/opt/sgxsdk/sdk_libs:/opt/intel/sgx-aesm-service/aesm
        fi
    else
        echo "✅ SGX SDK is already installed at /opt/sgxsdk where it also is the uninstall.sh script"
    fi

    # SGX objdump binutils
    echo "Downloading $sgx_objdump_url"
    wget $sgx_objdump_url &&
        tar xzf $objdump_name &&
        echo -e "Installing in /usr/bin:\n$(ls external/toolset/${distro_name%-server}/*)" &&
        cp -r external/toolset/${distro_name%-server}/* /usr/bin/ &&
        rm -rf external $objdump_name

    # DCAP driver
    echo "Downloading DCAP driver..."
    dcap_driver=$(curl -sSL $base_bin_url/driver_readme.txt | grep DCAP | awk '{print $NF}')
    if [ ! -z "${dcap_driver}" ] && [ -z "${NO_DCAP_DRIVER}" ] ; then
        wget $base_bin_url/$dcap_driver
        chmod u+x $dcap_driver
        echo "Installing DCAP driver..."
        ./$dcap_driver
        rm $dcap_driver
    else
        echo "[⚠️ ] Warning: could not find the name of DCAP driver on $base_bin_url/driver_readme.txt" >&2
    fi
}

# Debian-based dependencies installation
install_deb_deps()
{
    apt-get update
    apt-get -y install "${deb_deps_temp[@]}"

    check_sgx
    sgx_support=$?
    if [ $sgx_support -eq 1 ] || [ ! -z "${SGX_SIM}" ] ; then
        install_deb_sgx_psw
        install_sgx_sdk
    fi
    rm_temp_deps
}

rm_temp_deps()
{
    if [ ! -z "${TEMP_DEPENDENCIES}" ]; then
        apt-get -y remove "${deb_deps_temp[@]}" && apt-get -y autoremove
        rm -rf /var/lib/apt/lists/* && rm -rf /var/cache/apt/archives/*
    fi
}

################# main #################

if [ "$(id -u)" -eq 0 ] ; then
    echo "Running with superuser privileges..."
fi
if [ ! -z "${SGX_INSTALL_AS_ROOT}" ]; then
    echo "Environmental variable for running as root is set!"
fi
# Build as user
if [ "$(id -u)" -ne 0 ] || [ ! -z "${SGX_INSTALL_AS_ROOT}" ] ; then

    # For Debian based distros
    if [ -f "/etc/debian_version" ] ; then

        # Verifying dependencies
        verify_deps 'dpkg -s' "${deb_deps_sgx[@]}" "${deb_deps_dcap[@]}"
        DEPS_EXIT_STATUS=$?

        # If dependencies missing, installing them
        if ! (exit $DEPS_EXIT_STATUS) ; then
            missing_deps install_deb_deps
        fi
    else
        unrecognized_distro
    fi
    exit $?
else
    # Install dependencies as superuser
    echo "Installing dependencies..."
    if [ -f "/etc/debian_version" ] ; then
        install_deb_deps
    else
        unrecognized_distro
    fi
    EXIT_STATUS=$?
    if ! (exit $EXIT_STATUS) ; then
        exit $EXIT_STATUS
    fi
fi
