#!/bin/bash

ID=$(docker create try_gluon)
rm -f target/lambda.zip \
    && docker run --volume $(pwd):/outside --rm try_gluon cp -r /root/{Cargo.lock,public,src,target,try_gluon} /outside/target/ \
    && cp bootstrap target/ \
    && cd target \
    && sudo chown -R $USER target/dist \
    && rm -rf doc \
    && mv target/dist/doc ./ \
    && zip --recurse-paths lambda.zip bootstrap Cargo.lock public src try_gluon target

