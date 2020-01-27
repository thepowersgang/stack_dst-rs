#!/bin/bash

set -o errexit -o nounset

rev=$(git rev-parse --short HEAD)

git init
git config user.name "John Hodge"
git config user.email "tpg+travis@mutabah.net"

git remote add upstream "https://$GH_TOKEN@github.com/thepowersgang/stack_dst-rs.git"
git fetch upstream
git reset upstream/gh-pages

touch target/doc

git add -A .
git add -f target/doc/
git commit -m "rebuild pages at ${rev}"
git push -q upstream HEAD:gh-pages
