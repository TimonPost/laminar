FROM alpine:3.7

COPY laminar-tester /usr/local/bin/laminar-tester
RUN chmod ugo+x /usr/local/bin/laminar-tester
ENTRYPOINT [ "laminar-tester" ]