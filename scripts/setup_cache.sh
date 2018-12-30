#!/bin/sh

set -ex

if [ -z "$RUSTC_WRAPPER" ]; 
then 
    echo "No build caching setup!"
else
    SCCACHE_VERSION='sccache-0.2.7-x86_64-unknown-linux-musl'
    curl -L "https://github.com/mozilla/sccache/releases/download/0.2.7/$SCCACHE_VERSION.tar.gz" | tar -xvz
    mv $SCCACHE_VERSION/sccache .
    chmod +x ./sccache
    mv ./sccache /usr/local/bin/ 
fi
