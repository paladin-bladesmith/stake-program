#!/usr/bin/env zx
import 'zx/globals';
import {
  getProgramFolders,
  workingDirectory,
} from '../utils.mjs';

// Save external programs binaries to the output directory.
import './dump.mjs';

// Configure additional arguments here, e.g.:
// ['--arg1', '--arg2', ...cliArguments()]
const buildArgs = [
  '--features',
  'bpf-entrypoint',
  ...process.argv.slice(3),
];

// Build the programs.
await Promise.all(
  getProgramFolders().map(async (folder) => {
    const manifestPath = path.join(workingDirectory, folder, 'Cargo.toml');

    await $`cargo-build-sbf --manifest-path ${manifestPath} ${buildArgs}`;
  })
);
