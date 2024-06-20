#!/usr/bin/env zx
import 'zx/globals';
import {
  workingDirectory,
  getNightlyToolchain,
  getProgramFolders,
} from '../utils.mjs';

// Lint the programs using clippy.
for (const folder of getProgramFolders()) {
  cd(`${path.join(workingDirectory, folder)}`);
  await $`cargo +${getNightlyToolchain()} clippy ${process.argv.slice(3)}`;
}
