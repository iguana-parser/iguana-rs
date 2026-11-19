// The runtime's positions of a text in an editor: the line and the column in
// UTF-16 of a parser offset, which is in characters, and the offset at a line
// and a column (iguana-runtime/wasm, `cargo xtask runtime-wasm`). The wasm
// module's bytes are in the bundle, and the module is initialized when this
// file loads, so `TextPositions` is ready to use synchronously. The
// initialization is a side effect, so `sideEffects` in web-ui's package.json
// lists this file, and a bundler keeps it.
import { initSync, TextPositions } from "./runtime-wasm/iguana_runtime_wasm.js";
import base64 from "./runtime-wasm/iguana_runtime_wasm_bytes.js";

initSync({ module: Uint8Array.from(atob(base64), (c) => c.charCodeAt(0)) });

export { TextPositions };
