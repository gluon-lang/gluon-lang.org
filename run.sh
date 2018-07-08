set -ex

docker pull marwes/try_gluon

RUST_LOG=info docker run \
    --rm \
    -p 80:8080 \
    --name try_gluon_running \
    --env RUST_LOG \
    marwes/try_gluon
