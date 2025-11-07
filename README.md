# indico-md-rs — Indico-flavored Markdown renderer (workspace)

![PyPI version](https://flat.badgen.net/pypi/v/indico-md) ![npm version](https://flat.badgen.net/npm/v/indico-md)

![indico-comrak](https://github.com/pferreir/indico-md-rs/actions/workflows/indico-comrak.yml/badge.svg) ![indico-md-py](https://github.com/pferreir/indico-md-rs/actions/workflows/indico-md-py.yml/badge.svg) ![indico-md-wasm](https://github.com/pferreir/indico-md-rs/actions/workflows/indico-md-wasm.yml/badge.svg) ![pages](https://github.com/pferreir/indico-md-rs/actions/workflows/pages.yml/badge.svg)

This repository hosts the packages which implement and expose an Indico-flavored Markdown renderer for JS/WASM and Python.

[See the demo here](https://pferreir.github.io/indico-md-rs)

Workspace members
- 🦀 `indico-comrak`: Core Rust renderer and utilities (shared library used by other packages);
- 📦 `indico-md-wasm`: A tiny WebAssembly build of the renderer for browsers and Node.js (wasm-bindgen / wasm-pack);
- 🐍 `indico-md-py`: Python bindings for the renderer (PyO3 / maturin or a pip package);
- 🧪 `indico-md-test`: A simple web page which can be used to test the renderer.

## Markdown syntax

The following [comrak](https://github.com/kivikakk/comrak) extensions are used:
 * `strikethrough`
 * `header_ids`
 * `tagfilter`
 * `table`
 * `tasklist`
 * `alerts`
 * `autolink`
 * `math_code`
 * `math_dollars`
 * `underline`
 * `highlight` (contributed [by us](https://github.com/kivikakk/comrak/pull/672))

We also implement support for runtime auto-linking of strings based on regular expressions.


## Note
In applying the MIT license, CERN does not waive the privileges and immunities granted to it by virtue of its status as an Intergovernmental Organization or submit itself to any jurisdiction.