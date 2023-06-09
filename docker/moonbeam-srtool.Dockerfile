ARG SRTOOL_IMAGE_TAG

FROM paritytech/srtool:${SRTOOL_IMAGE_TAG}

USER root

RUN apt-get update && \
    apt-get install openssh-server -y

RUN groupmod -g 1020 builder && \
    usermod -u 1020 -g 1020 builder && \
    find /home/builder -uid 1001 -exec chown -v -h 1020 '{}' \; && \
    find /home/builder -gid 1001 -exec chgrp -v 1020 '{}' \;
USER 1020