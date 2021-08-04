#/bin/sh

# If you are in the scripts folder, go back to the root of the repository
FOLDER="${PWD##*/}"
if [[ "$FOLDER" == "scripts" ]]; then
  cd ..
fi

# Get relay binary
./scripts/get-alphanet-relay-image.sh

# Move to tests folder
cd tests

# Execute single test file provided
npm i
node_modules/.bin/mocha -r ts-node/register "para-tests/$1"

# if you were not in the scripts folder at the beginning, go back to the root of the repository
if [[ "$FOLDER" != "scripts" ]]; then
  cd ..
fi
