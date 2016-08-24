#!/bin/bash

set -o errexit -o nounset

: ${TRAVIS:?"This should only be run on Travis CI."}

rev=$(git rev-parse --short HEAD)

git init
git config user.name "Travis CI"
git config user.email "builder@travis"

git remote add upstream "https://$GH_TOKEN@github.com/francesca64/hotwatch.git"
git fetch upstream
git reset upstream/gh-pages

rm -rf docs
mv target/doc docs

git add -A docs
git commit -m "Updated docs for ${rev}"
git push --force --quiet upstream HEAD:gh-pages
