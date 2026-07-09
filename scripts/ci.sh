#!/bin/bash
set -eux
cargo test --all-features --all-targets

cargo run --bin try_gluon -- --port 3000 &
TRY_GLUON_PID=$!


until $(curl --output /dev/null --silent --fail http://localhost:3000); do
    if jobs %1; then
        printf '.'
        sleep 1
    else
        echo "ERROR: Server unexpectdly shutdown or could not start"
        exit 1
    fi
done

kill $TRY_GLUON_PID