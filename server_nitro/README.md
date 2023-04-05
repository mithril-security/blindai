Run proxy on host : 
``` 
sudo gvisor-tap-vsock/bin/gvproxy -debug -listen vsock://:1024 -listen unix:///tmp/network.sock
``` 
Start enclave

``` 
cd test_server
make
``` 

Connect to enclave via SSH :
``` 
ssh -o "StrictHostKeyChecking=no" -p 2222 root@127.0.0.1
```