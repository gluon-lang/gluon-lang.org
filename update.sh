#!/bin/bash
set -ex

cd $HOME/try_gluon

# The lock file may have been updated so reset it before pulling new changes
git checkout Cargo.lock
git pull origin master --ff-only
cargo update -p https://github.com/gluon-lang/gluon

./run.sh
