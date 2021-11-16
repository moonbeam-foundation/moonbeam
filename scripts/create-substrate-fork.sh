#!/bin/bash

BASE_BRANCH=${2:-"moonbeam-polkadot-v0.9.12"}
NEW_BRANCH=$1

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
  git clone --depth 1 git@github.com:purestake/$REPO.git -b $BASE_BRANCH
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
git checkout -b $NEW_BRANCH
