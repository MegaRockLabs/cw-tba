FROM ghcr.io/archway-network/archwayd-dev:v7.0.1-arm64v8

COPY ./entry-point.sh /data/entry-point.sh
RUN chmod +x /data/entry-point.sh

EXPOSE 9090
EXPOSE 1317
EXPOSE 26656
EXPOSE 26657

ENTRYPOINT ["/data/entry-point.sh"]