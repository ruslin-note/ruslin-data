#/bin/bash

set -e
bash tools/joplin-server/prepare-test-server.sh
cargo test -- --test-threads=1
