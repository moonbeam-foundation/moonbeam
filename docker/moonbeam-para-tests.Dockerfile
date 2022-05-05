# Inspired by Polkadot Dockerfile

FROM node:18-buster-slim
LABEL maintainer "alan@purestake.com"
LABEL description="This is a docker to run parachain tests"


RUN apt-get update
RUN apt-get install -y git

WORKDIR /moonbeam

COPY build ./build
COPY .github ./.github
COPY runtime ./runtime
COPY moonbeam-types-bundle ./moonbeam-types-bundle
COPY tests ./tests

RUN chmod uog+x build/polkadot
RUN chmod uog+x build/moonbeam
RUN cd moonbeam-types-bundle && npm install
RUN cd tests && npm install

RUN echo "#!/bin/sh" > run-para-tests.sh
RUN echo "cd /moonbeam/tests" > run-para-tests.sh
RUN echo "node_modules/.bin/mocha --exit -r ts-node/register 'para-tests/**/test-*.ts'" \
  >> run-para-tests.sh
RUN chmod uog+x run-para-tests.sh

CMD ["sh", "-c", "/moonbeam/run-para-tests.sh"]
