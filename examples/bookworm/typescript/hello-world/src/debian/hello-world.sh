#!/usr/bin/env sh

exec 2>&1

set -e

cd /usr/lib/hello-world-typescript
npm start --silent
