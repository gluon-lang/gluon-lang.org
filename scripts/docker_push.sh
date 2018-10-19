#!/bin/bash
set -ex

docker push marwes/try_gluon:dependencies
docker push marwes/try_gluon:builder
docker push marwes/try_gluon
