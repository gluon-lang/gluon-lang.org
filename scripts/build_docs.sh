#!/bin/sh

cargo doc -p https://github.com/gluon-lang/gluon --all-features && \
    mkdir -p dist/doc/nightly && \
    mv target/doc dist/doc/nightly/rust_doc && \
    cargo doc -p 'https://github.com/rust-lang/crates.io-index#gluon' --all-features && \
    mkdir -p dist/doc/crates_io && \
    mv target/doc dist/doc/crates_io/rust_doc
