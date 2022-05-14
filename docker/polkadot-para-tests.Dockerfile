# Node for Moonbase Parachains.
#
# Requires to run from repository root and to copy the binary in the build folder (part of the release workflow)

FROM node
LABEL maintainer "alan@purestake.com"
LABEL description="Node image use to run Moonbeam para-tests"

ARG HOST_UID=1001

RUN ((getent passwd $HOST_UID > /dev/null) || \
	  useradd -m -u $HOST_UID -U -s /bin/sh -d /polkadot polkadot) && \
	mkdir -p /polkadot/.local/share/polkadot && \
	chown -R $HOST_UID /polkadot && \
	ln -s /polkadot/.local/share/polkadot /data

RUN mkdir -p /binaries
COPY build/polkadot /binaries/
RUN chmod -R uog+rwX /binaries

USER $HOST_UID

WORKDIR /polkadot

ENTRYPOINT ["docker-entrypoint.sh"]
CMD [ "node" ]