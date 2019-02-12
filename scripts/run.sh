#!/bin/bash
set -ex

docker pull marwes/try_gluon
docker rm --force try_gluon_running || true

RUST_LOG=try_gluon=info,warn docker run \
    -p 80:80 \
    -p 443:443 \
    --name try_gluon_running \
    --env RUST_LOG \
    --env-file try_gluon.env \
    --mount source=letsencrypt,target=/etc/letsencrypt \
    --mount source=letsencrypt_log,target=/var/log/letsencrypt \
    marwes/try_gluon
