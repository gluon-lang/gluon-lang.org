#!/bin/bash
set -ex

cd $HOME/try_gluon

git pull origin master --ff-only
cargo update -p https://github.com/gluon-lang/gluon

./run.sh
