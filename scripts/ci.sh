#!/bin/bash
set -eux

cross build --target=x86_64-unknown-linux-musl ${RELEASE:-} --tests --bins --all-features

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
