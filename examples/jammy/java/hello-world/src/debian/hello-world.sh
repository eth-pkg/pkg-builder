#!/usr/bin/env bash

exec 2>&1

set -e
# Get the directory of the script
SCRIPT_DIR="/usr/lib/hello-world-java/bin"

java -classpath "$SCRIPT_DIR" com.example.HelloWorld.HelloWorld

