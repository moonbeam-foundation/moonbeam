# Node for Moonbase Parachains.
#
# Requires to run from repository root and to copy the binary in the build folder (part of the release workflow)

FROM phusion/baseimage:0.11
LABEL maintainer="alan@moonsonglabs.com"
LABEL description="Moonbeam network node. Supports Alphanet/Stagenet. Will support Moonriver and Moonbeam mainnet."
ARG PROFILE=release

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	rm -rf /usr/lib/python* && \
	useradd -m -u 1000 -U -s /bin/sh -d /moonbase-parachain moonbeam && \
	mkdir -p /moonbase-parachain/.local/share/moonbase-parachain && \
	chown -R moonbeam:moonbeam /moonbase-parachain && \
	ln -s /moonbase-parachain/.local/share/moonbase-parachain /data && \
	rm -rf /usr/bin /usr/sbin

USER moonbeam

COPY --chown=moonbeam build /moonbase-parachain
RUN chmod uog+x /moonbase-parachain/moonbeam

# 30333 for parachain p2p 
# 30334 for relaychain p2p 
# 9944 for Websocket and RPC call
# 9615 for Prometheus (metrics)
EXPOSE 30333 30334 9944 9615 

VOLUME ["/data"]

CMD ["/moonbase-parachain/moonbeam"]
