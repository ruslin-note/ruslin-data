#!/bin/bash

curl --data '{"action": "clearKeyValues"}' \
-H 'Content-Type: application/json' \
http://localhost:22300/api/debug
