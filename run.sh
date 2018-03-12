#!/bin/bash
set -x

yarn install
webpack
cargo update -p gluon_master
cargo build --release

OLD_GROUP_ID=$(ps x -o  "%p %r %y %x %c " | grep try_gluon | awk  '{print $2}')
if [ -n $OLD_GROUP_ID ]; then
	kill -- -$OLD_GROUP_ID
fi
RUST_LOG=info target/release/try_gluon --release &> output
