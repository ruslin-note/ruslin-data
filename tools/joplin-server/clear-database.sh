#!/bin/bash

curl --data '{"action": "clearDatabase"}' \
-H 'Content-Type: application/json' \
http://localhost:22300/api/debug
