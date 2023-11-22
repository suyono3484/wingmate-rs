FROM ubuntu:22.04

ADD target/debug/init /usr/local/bin/init
ADD docker/ /

RUN chmod ugo+x /etc/wingmate/services/one && chmod ugo+x /etc/wingmate/services/two.sh && \
    chmod ugo-x /etc/wingmate/services/three.sh

CMD [ "/usr/local/bin/init" ]