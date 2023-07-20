#!/bin/bash

if [[ -z "$1" ]]; then
  echo "usage: ./scripts/create-substrate-fork.sh <new_branch> [<base_branch>]";
  exit 1;
fi

NEW_BRANCH=$1
BASE_BRANCH=${2:-"moonbeam-polkadot-v0.9.17"}

mkdir -p scripts/tmp
cd scripts/tmp

REPOS=(
  substrate
  polkadot
  cumulus
  nimbus
  open-runtime-module-library
  crowdloan-rewards
  frontier
)

for REPO in ${REPOS[@]}; do
  git clone --depth 1 git@github.com:moonbeam-foundation/$REPO.git -b $BASE_BRANCH
  cd $REPO
  git checkout -b $NEW_BRANCH
  find . -name "Cargo.toml" -exec sed -i "s/\"$BASE_BRANCH\"/\"$NEW_BRANCH\"/g" {} \;
  git add .
  git commit -m "update git dependencies"
  git push -f origin $NEW_BRANCH
  cd ..
  rm -rf $REPO
done

cd ../..
rm -rf scripts/tmp
git checkout -b $NEW_BRANCH
