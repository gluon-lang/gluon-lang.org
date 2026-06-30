#!/bin/bash
set -eux

USE_CACHE=${1:-}
if [ "$USE_CACHE" == 'cache' ];
then
    EXTRA_BUILD_ARGS=(--network host --build-arg=RUSTC_WRAPPER=sccache)
else
    EXTRA_BUILD_ARGS=()
fi

BRANCH_NAME="${GITHUB_BASE_REF:-${GITHUB_REF_NAME:-}}"
if [ "$BRANCH_NAME" == 'master' ] || [ -n "${RELEASE:-}" ] ; then
    EXTRA_BUILD_ARGS+=(--build-arg 'RELEASE=--release' --build-arg 'CARGO_INCREMENTAL=0')
fi
echo ${EXTRA_BUILD_ARGS[@]+"${EXTRA_BUILD_ARGS[@]}"} \

CROSS_BASE_IMAGE=ghcr.io/cross-rs/x86_64-unknown-linux-musl:main

docker build \
    ${EXTRA_BUILD_ARGS[@]+"${EXTRA_BUILD_ARGS[@]}"} \
    --target dependencies \
    --tag marwes/try_gluon:dependencies \
    --cache-from $CROSS_BASE_IMAGE \
    --cache-from marwes/try_gluon:dependencies \
    .

if [ -n "${PUSH_DOCKER_CACHE:-}" ]; then
    docker push marwes/try_gluon:dependencies
fi

docker build \
    ${EXTRA_BUILD_ARGS[@]+"${EXTRA_BUILD_ARGS[@]}"} \
    --target builder \
    --tag marwes/try_gluon:builder \
    --cache-from $CROSS_BASE_IMAGE \
    --cache-from marwes/try_gluon:dependencies \
    --cache-from marwes/try_gluon:builder \
    .

if [ -n "${PUSH_DOCKER_CACHE:-}" ]; then
    docker push marwes/try_gluon:builder
fi

docker build \
    ${EXTRA_BUILD_ARGS[@]+"${EXTRA_BUILD_ARGS[@]}"} \
    --tag marwes/try_gluon \
    --cache-from $CROSS_BASE_IMAGE \
    --cache-from marwes/try_gluon:dependencies \
    --cache-from marwes/try_gluon:builder \
    --cache-from marwes/try_gluon \
    .

if [ -z ${BUILD_ONLY:-} ]; then
    docker run \
        --init \
        -it \
        --env=RUST_BACKTRACE \
        marwes/try_gluon:builder \
        cargo test --target=x86_64-unknown-linux-musl --all-features ${RELEASE:-}

    docker run \
        --rm \
        -p 80:80 \
        --name try_gluon_running \
        marwes/try_gluon \
        ./try_gluon &

    until $(curl --output /dev/null --silent --fail http://localhost); do
        if jobs %1; then
            printf '.'
            sleep 1
        else
            echo "ERROR: Server unexpectdly shutdown or could not start"
            exit 1
        fi
    done

    docker rm --force try_gluon_running || true
fi
