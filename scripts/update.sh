#!/bin/bash
set -ex

cd $HOME/try_gluon

git checkout .

git pull origin master --ff-only

docker system prune -f

./scripts/run.sh
