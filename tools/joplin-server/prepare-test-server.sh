#!/bin/bash

BASE_DIR=$(dirname $0)

bash $BASE_DIR/clear-database.sh
bash $BASE_DIR/create-test-users.sh
