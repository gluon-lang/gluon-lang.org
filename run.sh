set -x

docker build --tag try_gluon .

docker stop try_gluon_running || true
docker rm try_gluon_running || true

RUST_LOG=info docker run -p 80:8080 --name try_gluon_running --env RUST_LOG try_gluon
