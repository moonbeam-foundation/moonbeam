# Node for Moonbase Alphanet.
#
# Requires to run from repository root and to copy the binary in the build folder (part of the release workflow)

FROM debian:buster-slim
LABEL maintainer "alan@purestake.com"
LABEL description="Binary for Moonbeam Collator"

RUN useradd -m -u 1000 -U -s /bin/sh -d /moonbeam moonbeam && \
	mkdir -p /moonbeam/.local/share && \
	mkdir /data && \
	chown -R moonbeam:moonbeam /data && \
	ln -s /data /moonbeam/.local/share/moonbeam && \
	rm -rf /usr/bin /usr/sbin

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
