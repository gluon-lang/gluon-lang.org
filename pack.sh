#!/bin/bash
set -euo pipefail

rm -f target/lambda.zip

cp bootstrap target/

mkdir -p target/target

zip --recurse-paths target/lambda.zip \
  bootstrap \
  Cargo.lock \
  public \
  src

cp -r target/x86_64-unknown-linux-gnu/release/try_gluon target/
cd target
zip --recurse-paths lambda.zip dist doc try_gluon