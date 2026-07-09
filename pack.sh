#!/bin/bash
set -euo pipefail

rm -f target/lambda.zip

cp -r Cargo.lock public src target/x86_64-unknown-linux-gnu/release/try_gluon target/

cp bootstrap target/

cd target

# Move docs from webpack output to zip root if present
if [ -d target/dist/doc ]; then
  mv target/dist/doc ./doc
fi

zip --recurse-paths lambda.zip \
  bootstrap \
  Cargo.lock \
  public \
  src \
  try_gluon

