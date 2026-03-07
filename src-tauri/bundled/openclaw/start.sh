#!/bin/bash
DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"
export NODE_PATH="$DIR/node_modules"
if [ -f "dist/index.js" ]; then
    exec node dist/index.js "$@"
else
    echo "Error: OpenClaw not built. Run: npm install && npm run build"
    exit 1
fi
