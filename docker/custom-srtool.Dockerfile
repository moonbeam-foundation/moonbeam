ARG SRTOOL_IMAGE_TAG

FROM paritytech/srtool:${SRTOOL_IMAGE_TAG}

USER root

RUN apt-get update && \
    apt-get install openssh-server -y

USER 1001
