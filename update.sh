#!/bin/bash
set -ex

cd $HOME/try_gluon

git pull origin master --ff-only

./run.sh
