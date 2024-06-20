#!/usr/bin/env zx
import 'zx/globals';
import {
  workingDirectory,
  getNightlyToolchain,
  getProgramFolders,
} from '../utils.mjs';

// Format the programs.
for (const folder of getProgramFolders()) {
  cd(`${path.join(workingDirectory, folder)}`);
  await $`cargo +${getNightlyToolchain()} fmt ${process.argv.slice(3)}`;
}
