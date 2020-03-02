# required: subkey, grep, cut
# Extracts the babe and granpa public keys and send them to the keystore for two running nodes via RPC (ports 9933, 9934) 

echo "Amstrong babe:"
AMSTRONG_BABE=$(echo $(subkey inspect "//Amstrong" | grep "^  Public key (hex)" | cut -f2- -d:) | xargs)
echo $AMSTRONG_BABE
echo "Amstrong granpa:"
AMSTRONG_GRANPA=$(echo $(subkey --ed25519 inspect "//Amstrong" | grep "^  Public key (hex)" | cut -f2- -d:) | xargs)
echo $AMSTRONG_GRANPA
echo "Aldrin babe:"
ALDRIN_BABE=$(echo $(subkey inspect "//Aldrin" | grep "^  Public key (hex)" | cut -f2- -d:) | xargs)
echo $ALDRIN_BABE
echo "Aldrin granpa:"
ALDRIN_GRANPA=$(echo $(subkey --ed25519 inspect "//Aldrin" | grep "^  Public key (hex)" | cut -f2- -d:) | xargs)
echo $ALDRIN_GRANPA

curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d \
  '{
    "jsonrpc":"2.0",
    "id":1,
    "method":"author_insertKey",
    "params": [
      "babe",
      "//Amstrong",
      "'"$AMSTRONG_BABE"'"
    ]
  }'

curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d \
  '{
    "jsonrpc":"2.0",
    "id":1,
    "method":"author_insertKey",
    "params": [
      "gran",
      "//Amstrong",
      "'"$AMSTRONG_GRANPA"'"
    ]
  }'

curl http://localhost:9934 -H "Content-Type:application/json;charset=utf-8" -d \
  '{
    "jsonrpc":"2.0",
    "id":1,
    "method":"author_insertKey",
    "params": [
      "babe",
      "//Aldrin",
      "'"$ALDRIN_BABE"'"
    ]
  }'

curl http://localhost:9934 -H "Content-Type:application/json;charset=utf-8" -d \
  '{
    "jsonrpc":"2.0",
    "id":1,
    "method":"author_insertKey",
    "params": [
      "gran",
      "//Aldrin",
      "'"$ALDRIN_GRANPA"'"
    ]
  }'
