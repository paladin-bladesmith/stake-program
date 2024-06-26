#!/usr/bin/env zx
import 'zx/globals';
import {
  getClippyToolchain,
  getProgramFolders,
  getToolchainArg,
  workingDirectory,
} from '../utils.mjs';

const toolchain = getToolchainArg(getClippyToolchain());
// Format the programs using rust fmt.
for (const folder of getProgramFolders()) {
  cd(`${path.join(workingDirectory, folder)}`);
  await $`cargo ${toolchain} fmt ${process.argv.slice(3)}`;
}
