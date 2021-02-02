# Node for Moonbase Alphanet.
#
# Requires to run from repository root and to copy the binary in the build folder (part of the release workflow)

FROM phusion/baseimage:0.11
LABEL maintainer "alan@purestake.com"
LABEL description="Binary for Moonbeam Collator"
ARG PROFILE=release

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	rm -rf /usr/lib/python* && \
	useradd -m -u 1000 -U -s /bin/sh -d /moonbeam moonbeam && \
	mkdir -p /moonbeam/.local/share/moonbeam && \
	chown -R moonbeam:moonbeam /moonbeam && \
	ln -s /moonbeam/.local/share/moonbeam /data && \
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

ENTRYPOINT ["/moonbeam/moonbeam"]
