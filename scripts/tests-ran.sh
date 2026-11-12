#!/usr/bin/env bash
# Usage: scripts/tests-ran.sh <test-target|bin> <filter>
# Fails when cargo test fails or when no test matched the filter, so a ticket's
# verify command fails until the named tests exist and pass.
set -u
if [ "$1" = bin ]; then target=(--bin slopr); else target=(--test "$1"); fi
out=$(cargo test "${target[@]}" -- "$2" 2>&1)
status=$?
echo "$out" | tail -n 20
[ $status -eq 0 ] || exit 1
echo "$out" | grep -Eq 'test result: ok\. [1-9][0-9]* passed' || { echo "no test matched '$2'"; exit 1; }
