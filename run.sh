#!/bin/bash

webpack
cargo build --release

OLD_GROUP_ID=$(ps x -o  "%p %r %y %x %c " | grep cargo | awk  '{print $2}')
if [ -n $OLD_GROUP_ID ]; then
	kill -- -$OLD_GROUP_ID
fi
RUST_LOG=info cargo run --release &> output
