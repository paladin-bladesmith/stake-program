#!/usr/bin/env zx
import "zx/globals";
import {
  getClippyToolchain,
  getProgramFolders,
  getToolchainArg,
  workingDirectory,
} from "../utils.mjs";

const toolchain = getToolchainArg(getClippyToolchain());
// Lint the programs using rust fmt and clippy.
for (const folder of getProgramFolders()) {
  cd(`${path.join(workingDirectory, folder)}`);
  await $`cargo ${toolchain} fmt --check ${process.argv.slice(3)}`;
  await $`cargo ${toolchain} clippy ${process.argv.slice(3)}`;
}
