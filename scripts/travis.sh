#!/bin/bash
set -eux

if [ "$1" == 'cache' ];
then
    docker build \
        --pull \
        --network host \
        --build-arg=RUSTC_WRAPPER=./sccache \
        --cache-from marwes/try_gluon_builder \
        --tag marwes/try_gluon_builder \
        --target try_gluon_builder \
        .
else
    docker build \
        --pull \
        --cache-from marwes/try_gluon_builder \
        --tag marwes/try_gluon_builder \
        --target try_gluon_builder \
        .
fi

docker run \
    --init \
    -it \
    --env=RUST_BACKTRACE marwes/try_gluon_builder \
    cargo test --release

docker build \
    --pull \
    --cache-from marwes/try_gluon,marwes/try_gluon_builder \
    --tag marwes/try_gluon \
    .
