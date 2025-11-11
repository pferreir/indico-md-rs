# indico-md — Python bindings for the Indico Markdown renderer

`indico-md` exposes a small, zero-dependency Python interface to the Rust-based
Indico-flavored Markdown renderer. It is a native extension built with PyO3 / maturin. It supports runtime autolinking via regular-expressions.

## Installation

From PyPI:
```bash
pip install indico-md
```

Build and install locally (recommended for development):
```bash
pip install maturin
maturin develop --release
maturin build --release
```

## Quick usage

Python API:
```python
to_html(md_source: str, link_rules: Dict[str, str]) -> str
to_unstyled_html(md_source: str) -> str
```

Example:
```python
import indico_md

md = "See TKT1234567 and gh:123"
rules = {
    r"\bTKT(\d{7})\b": "https://tkt.sys/{1}",
    r"\bgh:(\d+)\b": "https://github.com/indico/indico/issues/{1}",
}

html = indico_md.to_html(md, rules)
print(html)
```

## Tests
Run them with:
```bash
pip install -e '.[test]'
pytest
```
