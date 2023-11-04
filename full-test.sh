#/bin/bash

set -e
bash tools/joplin-server/prepare-test-server.sh
export RUSTFLAGS="--cfg uuid_unstable"
cargo test -- --test-threads=1
