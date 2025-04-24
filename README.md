# De Wiggers-Graaf

This is a graph based solver for the Klotski game in the Wiggers family.

[Play here!](https://mercotui.com/wiggers-graaf)

## Building

This depends on wasm-pack,
use the following command to install it globally on your system, and then use it:

```bash
cargo install wasm-pack
wasm-pack build --target web --out-dir src/web/pkg
```

Then load `src/web/index.html` in your webbrowser!
