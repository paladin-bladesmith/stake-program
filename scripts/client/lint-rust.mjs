#!/usr/bin/env zx
import 'zx/globals';
import { getClippyToolchain, workingDirectory } from '../utils.mjs';

// Check the client using clippy.
cd(path.join(workingDirectory, 'clients', 'rust'));
await $`cargo ${getClippyToolchain()} clippy ${process.argv.slice(3)}`;
