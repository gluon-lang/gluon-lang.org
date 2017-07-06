#!/bin/bash
cargo build --release && RUST_LOG=info cargo run --release &> output
