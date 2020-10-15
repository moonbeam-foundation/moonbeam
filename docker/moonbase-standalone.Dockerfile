# Node for Moonbase Alphanet. 
# 
# Requires to run from repository root and to copy the binary in the build folder (part of the release workflow)

FROM phusion/baseimage:0.11
LABEL maintainer "alan@purestake.com"
LABEL description="this is the standalone node running Moonbase"
ARG PROFILE=release

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	rm -rf /usr/lib/python* && \
	useradd -m -u 1000 -U -s /bin/sh -d /moonbase moonbeam && \
	mkdir -p /moonbase/.local/share/moonbase && \
	chown -R moonbeam:moonbeam /moonbase && \
	ln -s /moonbase/.local/share/moonbase /data && \
	rm -rf /usr/bin /usr/sbin


USER moonbeam

COPY --chown=moonbeam build/standalone /moonbase
RUN chmod uog+x /moonbase/moonbase-standalone

# 30333 for p2p traffic
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 30333 9933 9944 9615

CMD ["/moonbase/moonbase-standalone", \
	"--dev" \
	"--tmp" \
	"--charlie" \
    "--port","30333", \
    "--rpc-port","9933", \
    "--ws-port","9944", \
]