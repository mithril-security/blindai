FROM ubuntu:jammy

ARG ssh_pub_key

RUN mkdir -p /root/.ssh \
    && chmod 0700 /root/.ssh \
    && echo "$ssh_pub_key" > /root/.ssh/authorized_keys \
    && apt-get update \
    && apt-get install -y openssh-server \
    && ssh-keygen -A \
    && echo "PasswordAuthentication no" >> /etc/ssh/sshd_config

COPY nitriding /
COPY start.sh /

CMD ["/start.sh"]
