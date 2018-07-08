
# The lock file may have been updated so reset it before pulling new changes
git pull origin master --ff-only && \
    cargo update -p https://github.com/gluon-lang/gluon && \
    cargo run --release
