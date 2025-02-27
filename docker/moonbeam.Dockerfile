# Moonbeam Binary
#
# Requires to run from repository root and to copy the binary in the build folder (part of the release workflow)

FROM debian:stable AS builder

RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates

FROM debian:stable-slim
LABEL maintainer="alan@moonsonglabs.com"
LABEL description="Moonbeam Binary"

RUN useradd -m -u 1000 -U -s /bin/sh -d /moonbeam moonbeam && \
	mkdir -p /moonbeam/.local/share && \
	mkdir /data && \
	chown -R moonbeam:moonbeam /data && \
	ln -s /data /moonbeam/.local/share/moonbeam && \
	rm -rf /usr/sbin

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

USER moonbeam

COPY --chown=moonbeam build/* /moonbeam
RUN chmod uog+x /moonbeam/moonbeam*

# 30333 for parachain p2p
# 30334 for relaychain p2p
# 9944 for Websocket & RPC call
# 9615 for Prometheus (metrics)
EXPOSE 30333 30334 9944 9615

VOLUME ["/data"]

ENTRYPOINT ["/moonbeam/moonbeam"]
