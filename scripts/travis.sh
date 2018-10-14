#!/bin/bash
set -eux

USE_CACHE=${1:-}
if [ "$USE_CACHE" == 'cache' ];
then
    EXTRA_BUILD_ARGS=(--network host --build-arg=RUSTC_WRAPPER=./sccache)
else
    EXTRA_BUILD_ARGS=()
fi
docker build \
    "${EXTRA_BUILD_ARGS[@]}" \
    --cache-from marwes/try_gluon:builder \
    --tag marwes/try_gluon:builder \
    --target builder \
    .

docker run \
    --init \
    -it \
    --env=RUST_BACKTRACE \
    marwes/try_gluon:builder \
    cargo test --release

docker build \
    "${EXTRA_BUILD_ARGS[@]}" \
    --cache-from marwes/try_gluon \
    --cache-from marwes/try_gluon:builder \
    --tag marwes/try_gluon \
    .
