#!/bin/bash
pwd
ls -la
ls -la /rammoonbeam
mkdir -p /rammoonbeam/tests
ln -s /moonbeam/tests/* /rammoonbeam/tests/
cd /rammoonbeam/tests/
node node_modules/.bin/mocha --exit -r ts-node/register $@