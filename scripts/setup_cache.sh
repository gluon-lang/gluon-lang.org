set -ex

if [ -z "$RUSTC_WRAPPER" ]; 
then 
    echo "No build chaching setup!"
else
    SCCACHE_VERSION='sccache-0.2.7-x86_64-unknown-linux-musl'
    wget "https://github.com/mozilla/sccache/releases/download/0.2.7/$SCCACHE_VERSION.tar.gz"
    tar -xvzf "$SCCACHE_VERSION.tar.gz"
    mv $SCCACHE_VERSION/sccache .
    chmod +x ./sccache
fi
