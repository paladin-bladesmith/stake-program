#!/usr/bin/env zx
import 'zx/globals';
import {
  workingDirectory,
  getRustfmtToolchain,
  getProgramFolders,
} from '../utils.mjs';

// Format the programs.
for (const folder of getProgramFolders()) {
  cd(`${path.join(workingDirectory, folder)}`);
  await $`cargo ${getRustfmtToolchain()} fmt ${process.argv.slice(3)}`;
}
