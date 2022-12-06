#!/bin/bash

curl --data '{"action": "createTestUsers", "count": 10, "fromNum": 1}' \
-H 'Content-Type: application/json' \
http://localhost:22300/api/debug
