# indico-md-wasm 📦

A tiny WebAssembly build of the indico markdown renderer for use in browsers and Node.js.

## Features
- Fast, Rust-based markdown rendering compiled to WASM;
- Minimal JavaScript glue via wasm-bindgen;
- Works in browser (ESM) and Node (CommonJS / ESM);
- Zero-dependency runtime for embedding in web apps.

## Quickstart

### Prerequisites
- Rust toolchain (rustup)
- wasm-pack (or cargo + wasm-bindgen)
- Node.js (for Node usage) or a static server for browser

### Build locally
```bash
# build a package suitable for bundlers (webpack/rollup)
wasm-pack build --release --target bundler

# OR for direct browser usage
wasm-pack build --release --target web
```

Browser usage (ESM)
```html
<script type="module">
  import init, { toHtml } from "./pkg/indico_md_wasm.js";

  await init(); // loads and initializes the .wasm

  // pass an array of [RegExp, string] pairs for custom link rules,
  // or an empty array if you don't need custom rules.
  const html = toHtml("# Hello\nThis is indico-md-wasm.", []);
  document.body.innerHTML = html;
</script>
```

Node usage (ESM)
```js
import init, { toHtml } from "./pkg/indico_md_wasm.js";
await init();
console.log(toHtml("**bold** text", []));
```

Link rules example
```js
const rules = [
  [/^#(\d+)$/, "https://example.com/issues/$1"],
  [/^@(\w+)$/, "https://example.com/users/$1"]
];

const html = toHtml("See #123 and @user", rules);
```

API (exports)
- (default) `init(): Promise<void>` — initializes the WASM module
- `toHtml(source: string, rules: Array): string` — converts Indico-flavored markdown to HTML; `rules` is a JS array of `[RegExp, string]` pairs (use `[]` when none)
- `toUnstyledHtml(source: string): string` — converts Indico-flavored markdown to HTML, removing all formatting, links and images (i.e. only paragraphs and line breaks)

### Tests
```bash
wasm-pack test --node
```
