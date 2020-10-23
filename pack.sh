#!/bin/bash

rm -f target/lambda.zip \
    && docker run --volume $(pwd):/outside --rm try_gluon cp -r /root/{Cargo.lock,public,src,target,try_gluon}  /outside/target/ \
    && cp bootstrap target/ \
    && cd target \
    && zip --recurse-paths lambda.zip bootstrap Cargo.lock public src target try_gluon

