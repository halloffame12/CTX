#!/usr/bin/env node
"use strict";

/**
 * ctx launcher — ctxai-cli
 *
 * The real `ctx` is a native Rust binary shipped in an optional platform
 * package (ctxai-<platform>-<arch>). npm installs only the matching one.
 * This file locates that binary and executes it with the same argv.
 *
 * No network access, no arbitrary shell execution, no eval.
 */

const { spawn } = require("node:child_process");

function locateBinary() {
  const platform = process.platform; // linux | darwin | win32
  const arch = process.arch; // x64 | arm64
  // npm's spam detector rejects the name "ctxai-win32-arm64", so the
  // win32/arm64 binary ships as "ctxai-windows-arm64" instead.
  const pkg = platform === "win32" && arch === "arm64" ? "ctxai-windows-arm64" : `ctxai-${platform}-${arch}`;

  // Platform packages export "./binary" -> the native binary path.
  try {
    return require.resolve(`${pkg}/binary`, { paths: [__dirname] });
  } catch {
    // provide a deterministic, user-facing failure
    const err = new Error(
      `ctx: no binary found for ${platform}/${arch}. ` +
        `Supported platforms: linux, darwin, win32 × x64, arm64. ` +
        `Reinstall with "npm install -g ctxai-cli" so the matching platform package installs.`
    );
    throw err;
  }
}

if (require.main === module) {
  let bin;
  try {
    bin = locateBinary();
  } catch (err) {
    console.error(err.message);
    process.exit(1);
  }

  const child = spawn(bin, process.argv.slice(2), {
    stdio: "inherit",
    windowsHide: true,
  });

  child.on("error", (err) => {
    console.error(`ctx: failed to start native binary: ${err.message}`);
    process.exit(1);
  });

  child.on("close", (code) => {
    process.exit(code === null ? 1 : code);
  });
}