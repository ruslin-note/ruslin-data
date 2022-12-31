#/bin/bash

set -e

# rm -rf database.sqlite
diesel migration run --database-url=database.sqlite
cargo fmt
