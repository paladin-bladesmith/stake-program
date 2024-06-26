#!/usr/bin/env zx
import "zx/globals";
import {
  getClippyToolchain,
  getToolchainArg,
  workingDirectory,
} from "../utils.mjs";

const toolchain = getToolchainArg(getClippyToolchain());
// Check the client using rust fmt and clippy.
cd(path.join(workingDirectory, "clients", "rust"));
await $`cargo ${toolchain} fmt --check ${process.argv.slice(3)}`;
await $`cargo ${toolchain} clippy ${process.argv.slice(3)}`;
