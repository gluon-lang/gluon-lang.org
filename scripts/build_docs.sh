#!/bin/sh

cargo doc -p https://github.com/gluon-lang/gluon --all-features && \
    mkdir -p target/dist/doc/nightly && \
    cp -r target/doc target/dist/doc/nightly/rust_doc && \
    cargo doc -p 'https://github.com/rust-lang/crates.io-index#gluon' --all-features && \
    mkdir -p target/dist/doc/crates_io && \
    mv target/doc target/dist/doc/crates_io/rust_doc
