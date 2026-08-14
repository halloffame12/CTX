#!/bin/bash
export PATH=/usr/local/cargo/bin:$PATH
cd /qa/qa-project
for c in impact context changed diff mcp; do
  echo "===== ctx $c --help ====="
  /repo/target/release/ctx $c --help 2>&1
  echo
done