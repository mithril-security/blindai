Run proxy on host : 
``` 
sudo gvisor-tap-vsock/bin/gvproxy -debug -listen vsock://:1024 -listen unix:///tmp/network.sock
``` 
EXPOSE :8443 with : 
``` 
sudo curl  --unix-socket /tmp/network.sock http:/unix/services/forwarder/expose -X POST -d '{"local":"127.0.0.1:8443","remote":"192.168.127.2:8443"}'
``` 
Show all ports exposed : 
```
sudo curl  --unix-socket /tmp/network.sock http:/unix/services/forwarder/all | jq .
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