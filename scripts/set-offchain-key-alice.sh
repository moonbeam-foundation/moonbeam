echo "Alice babe:"
ALICE_BABE=$(echo $(subkey inspect "//Alice" | grep "^  Public key (hex)" | cut -f2- -d:) | xargs)
echo $ALICE_BABE

curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d \
  '{
    "jsonrpc":"2.0",
    "id":1,
    "method":"author_insertKey",
    "params": [
      "mbst",
      "//Alice",
      "'"$ALICE_BABE"'"
    ]
  }'
