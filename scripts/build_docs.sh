#!/bin/bash

declare -a PROJECTS=(
    gluon_codegen
    gluon_base
    gluon_parser
    gluon_check
    gluon_completion
    gluon_vm
    gluon_format
    gluon
    # gluon_c-api
    gluon_doc
    # gluon_repl
)

NIGHTLY_ARGS=()
for CRATE in ${PROJECTS[@]}; do
    NIGHTLY_ARGS+=("-p https://github.com/gluon-lang/gluon#${CRATE}")
done

cargo doc --no-deps ${NIGHTLY_ARGS[@]} --all-features && \
    mkdir -p target/dist/doc/nightly && \
    cp -r target/doc target/dist/doc/nightly/rust_doc

ARGS=()
for CRATE in ${PROJECTS[@]}; do
    ARGS+=("-p https://github.com/rust-lang/crates.io-index#${CRATE}")
done
cargo doc --no-deps ${ARGS[@]} --all-features && \
    mkdir -p target/dist/doc/crates_io && \
    mv target/doc target/dist/doc/crates_io/rust_doc
