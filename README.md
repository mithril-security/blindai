# Generate CA and certificates with OpenSSL CLI : 
```
## Create a CA
# Generate an ECDSA secp256p1 private key
openssl ecparam -genkey -name prime256v1 -out ca.key
# Create a CA 
openssl req -x509 -new -nodes -subj "/C=US/O=_Development CA/CN=Development certificates" -key ca.key -sha256 -days 3650 -out ca.pem 

## Create server certificate
# Generate server private key
openssl ecparam -genkey -name prime256v1 -out server.key

# Create a certificate signing request (CSR)
openssl req -new -subj "/C=US/O=Local Development/CN=localhost" -key "server.key" -out "server.csr"
# Use the previously created CA to sign the CSR to get the server certificate
# 
openssl x509 -req \
    -in "server.csr" \
    -extfile "server.ext" \
    -CA ca.pem \
    -CAkey ca.key \
    -CAcreateserial \
    -out "server.pem" \
    -days 365 \
    -sha256
# Convert key into PCKS8 DER format 
openssl pkcs8 -topk8 -inform PEM -outform PEM -in server.key -out server2.key -nocrypt
```
# Run 

```
# Launch server in a terminal
$ cargo run --target=x86_64-fortanix-unknown-sgx 
Now listening on port 9975
# In another terminal make a request
# -k option is to ignore certificate check since we signed the certificate
# with our own custom CA
$ curl -k https://localhost:9975
hello world% 
``` 