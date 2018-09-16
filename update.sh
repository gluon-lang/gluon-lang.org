#!/bin/bash
set -ex

cd $HOME/try_gluon

git pull origin master --ff-only

docker system prune -f

./run.sh
