#!/usr/bin/env sh

set -euo pipefail

mkdir -p target/test-db
echo 0 > target/test-db/$NEXTEST_RUN_ID
