# Dummy makefile, will call the host and enclave makefile when requested.

SRC_U = inference-server/scheduler/untrusted/
SRC_T = inference-server/scheduler/trusted/

CUSTOM_EDL_PATH != realpath 'sdk/edl'
CUSTOM_COMMON_PATH != realpath 'sdk/common'
XARGO_PATH != realpath 'sdk/xargo'
ENCLAVE_METADATA = inference-server/scheduler/trusted/enclave.meta.txt

export CUSTOM_EDL_PATH CUSTOM_COMMON_PATH XARGO_PATH

SSL_CERT_KEY = bin/tls/host_server.pem bin/tls/host_server.key

# Compilation process, will call the appropriate makefiles.

all: host enclave policy.toml

init: check-and-reinit-submodules $(SSL_CERT_KEY)

# Generate SSL key and certificate for RPC secure channel
$(SSL_CERT_KEY):
	@echo "\033[32mGenerating SSL key...\033[0m"
	@mkdir -p bin/tls
	@openssl req -newkey rsa:2048 -nodes -keyout bin/tls/host_server.key  \
		-x509 -days 365 -out bin/tls/host_server.pem \
		-subj "/C=FR/CN=blindai-srv"
	@cp bin/tls/host_server.pem .

$(ENCLAVE_METADATA): enclave

policy.toml: $(ENCLAVE_METADATA)
	python3 tools/gen_enclave_info_toml.py < $(ENCLAVE_METADATA) > policy.toml

check-and-reinit-submodules:
	@if git submodule status | egrep -q '^[-]|^[+]' ; then \
		echo "INFO: Need to reinitialize git submodules"; \
		git submodule update --init; \
	fi

host:
	@echo "\033[32mRequest to compile the host part...\033[0m"
	@make -C $(SRC_U)

enclave:
	@echo "\033[32mRequest to compile the enclave part...\033[0m"
	@make -C $(SRC_T)

clean: clean_client
	@make -C $(SRC_U) clean
	@make -C $(SRC_T) clean

fclean:
	@make -C $(SRC_U) fclean
	@make -C $(SRC_T) fclean

clean_host:
	@make -C $(SRC_U) clean

clean_enclave:
	@make -C $(SRC_T) clean

fclean_host:
	@make -C $(SRC_U) fclean

fclean_enclave:
	@make -C $(SRC_T) fclean

re_host: fclean_host host

re_enclave: fclean_enclave enclave

re: fclean all

# Dummy rules to let make know that those rules are not files.

.PHONY: init check-and-reinit-submodules host enclave client clean clean_host clean_enclave clean_client fclean_host fclean_enclave fclean re re_host re_enclave