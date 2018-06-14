set -ex

docker build --tag try_gluon .

docker rm --force try_gluon_running

RUST_LOG=info docker run \
    --rm \
    -p 80:8080 \
    --name try_gluon_running \
    --env RUST_LOG try_gluon \
    --volume cargo:/usr/local/cargo \
    --volume target:/usr/src/try_gluon/target
