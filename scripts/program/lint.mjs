#!/usr/bin/env zx
import 'zx/globals';
import {
  workingDirectory,
  getClippyToolchain,
  getProgramFolders,
} from '../utils.mjs';

// Lint the programs using clippy.
for (const folder of getProgramFolders()) {
  cd(`${path.join(workingDirectory, folder)}`);
  await $`cargo ${getClippyToolchain()} clippy ${process.argv.slice(3)}`;
}
