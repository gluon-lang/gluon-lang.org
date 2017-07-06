#!/bin/bash

cargo build --release

OLD_GROUP_ID=$(ps x -o  "%p %r %y %x %c " | grep cargo | awk  '{print $2}')
kill -- -$OLD_GROUP_ID
RUST_LOG=info cargo run --release &> output
