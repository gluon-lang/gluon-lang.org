#!/bin/bash
set -eux

USE_CACHE=${1:-}
if [ "$USE_CACHE" == 'cache' ];
then
    docker build \
        --pull \
        --network host \
        --build-arg=RUSTC_WRAPPER=./sccache \
        --cache-from marwes/try_gluon:builder \
        --tag marwes/try_gluon:builder \
        --target builder \
        .
else
    docker build \
        --pull \
        --cache-from marwes/try_gluon:builder \
        --tag marwes/try_gluon:builder \
        --target builder \
        .
fi

docker run \
    --init \
    -it \
    --env=RUST_BACKTRACE \
    marwes/try_gluon:builder \
    cargo test --release

docker build \
    --pull \
    --cache-from marwes/try_gluon \
    --cache-from marwes/try_gluon:builder \
    --tag marwes/try_gluon \
    .
