import "zx/globals";
import { getClippyToolchain, getRustfmtToolchain } from "./utils.mjs";

// RUST_FMT_TOOLCHAIN
const fmtToolchain =
  getCargoMetadata(folder).scripts?.rustfmt?.toolchain?.channel;
echo(`RUST_FMT_TOOLCHAIN="${fmtToolchain ?? ""}"`);
// CLIPPY_TOOLCHAIN
const clippyToolchain =
  getCargoMetadata(folder).scripts?.clippy?.toolchain?.channel;
echo(`CLIPPY_TOOLCHAIN="${clippyToolchain ?? ""}"`);
