#!/bin/bash
cd $HOME/try_gluon && git pull origin master --ff-only && RUST_LOG=info cargo run --release &> output
