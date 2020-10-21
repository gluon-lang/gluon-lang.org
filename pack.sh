#!/bin/bash

rm target/lambda.zip \
    && docker run --volume $(pwd):/root/mount --rm try_gluon cp /root/try_gluon /root/mount/target/try_gluon \
    && cp bootstrap target/ \
    && cd target \
    && zip lambda.zip bootstrap try_gluon

