# Node for Moonbase Alphanet.
#
# Requires to run from repository root and to copy the binary in the build folder (part of the release workflow)

FROM docker.io/library/ubuntu:20.04 AS builder

RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates

FROM debian:buster-slim
LABEL maintainer "alan@purestake.com"
LABEL description="Binary for Moonbeam Collator"

RUN useradd -m -u 1000 -U -s /bin/sh -d /moonbeam moonbeam && \
	mkdir -p /moonbeam/.local/share && \
	mkdir /data && \
	chown -R moonbeam:moonbeam /data && \
	ln -s /data /moonbeam/.local/share/moonbeam

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

USER moonbeam

COPY --chown=moonbeam build/moonbeam /moonbeam/moonbeam
RUN chmod uog+x /moonbeam/moonbeam

# 30333 for parachain p2p 
# 30334 for relaychain p2p 
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 30333 30334 9933 9944 9615 

VOLUME ["/data"]

ENTRYPOINT ["/moonbeam/moonbeam"]
