set -ex

docker pull marwes/try_gluon
docker rm --force try_gluon_running || true

RUST_LOG=try_gluon=info,warn docker run \
    --rm \
    -p 80:80 \
    -p 443:443 \
    --name try_gluon_running \
    --env RUST_LOG \
    --env-file try_gluon.env \
    marwes/try_gluon
