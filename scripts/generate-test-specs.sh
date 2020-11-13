#!/bin/bash
source scripts/_init_var.sh

if [ ! -f "$STANDALONE_BINARY" ]; then
    echo "Standalone binary $STANDALONE_BINARY is missing"
    echo "Please run: cargo build --release -p moonbase-standalone"
    exit 1
fi

$STANDALONE_BINARY build-spec \
  --disable-default-bootnode \
  | grep '\"code\"' \
  | head -n1 \
  > $STANDALONE_SPEC_TMP
echo $STANDALONE_SPEC_TMP generated

TEST_SPEC_TEMPLATE="tests/moonbeam-test-specs/templates/simple-specs-template.json"
TEST_SPEC_PLAIN="tests/moonbeam-test-specs/simple-specs.json"

echo "Using $TEST_SPEC_TEMPLATE..."
sed -e "/\"<runtime_code>\"/{r $STANDALONE_SPEC_TMP" -e 'd;}'  $TEST_SPEC_TEMPLATE\
  > $TEST_SPEC_PLAIN
echo $TEST_SPEC_PLAIN generated
