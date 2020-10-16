#!/bin/bash

cp bootstrap /lib/x86_64-linux-gnu/libssl.so* target/release/ \
    && cd target/release \
    && zip lambda.zip libssl.so* bootstrap try_gluon

