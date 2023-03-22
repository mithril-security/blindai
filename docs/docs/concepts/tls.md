# Securely communicating with BlindAI
__________________________________________

## A Reverse-proxy for the Unsecure port 
__________________________________________

The unsecure port was made to run only on HTTP. For that case, we highly recommand placing a reverse-proxy, that takes care of the TLS encryption for the unsecure port 9923 between the client and the server. 

## How to choose
__________________________________________

Multiple reverse-proxies that are usable with are available in use. 
Apache, Nginx and Caddy are good examples. 

[https://httpd.apache.org/docs/2.4/howto/reverse_proxy.html](https://httpd.apache.org/docs/2.4/howto/reverse_proxy.html)


[https://docs.nginx.com/nginx/admin-guide/web-server/reverse-proxy/](https://docs.nginx.com/nginx/admin-guide/web-server/reverse-proxy/)

[https://caddyserver.com/docs/quick-starts/reverse-proxy](https://caddyserver.com/docs/quick-starts/reverse-proxy)