#!/usr/bin/env zx
import 'zx/globals';
import { getNightlyToolchain, workingDirectory } from '../utils.mjs';

// Check the client using nightly clippy.
cd(path.join(workingDirectory, 'clients', 'rust'));
await $`cargo +${getNightlyToolchain()} clippy ${process.argv.slice(3)}`;
