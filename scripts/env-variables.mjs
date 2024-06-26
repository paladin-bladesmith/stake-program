import "zx/globals";
import { getClippyToolchain, getRustfmtToolchain } from "./utils.mjs";

echo(`RUST_FMT_TOOLCHAIN="${getRustfmtToolchain()}"`);
echo(`RUST_CLIPPY_TOOLCHAIN="${getClippyToolchain()}"`);
