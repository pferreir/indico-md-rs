import pytest
import indico_md

def test_output_ok():
    source = """## TEST
 * TKT1234567: solved
 * Still checking gh:123
 * [gh:124](https://somewhere.else) shouldn't be autolinked
"""
    rules = {
        r"\bTKT(\d{7})\b": "https://tkt.sys/{1}",
        r"\bgh:(\d+)\b": "https://github.com/indico/indico/issues/{1}"
    }
    result = """<h2><a href="#test" aria-hidden="true" class="anchor" id="indico-md-test"></a>TEST</h2>
<ul>
<li><a href="https://tkt.sys/1234567" title="TKT1234567">TKT1234567</a>: solved</li>
<li>Still checking <a href="https://github.com/indico/indico/issues/123" title="gh:123">gh:123</a></li>
<li><a href="https://somewhere.else">gh:124</a> shouldn't be autolinked</li>
</ul>
"""
    assert indico_md.to_html(source, rules) == result

def test_exceptions():
    source = "TEST"
    rules = {
        r"\bTKT(\d{7})\b": "https://tkt.sys/{1}",
        r"\bgh:(\d+)\b": 1234
    }
    with pytest.raises(TypeError):
        indico_md.to_html(source, rules)

    rules = {
        r"\bTKT(\d{7})\b": "https://tkt.sys/{1}",
        r"(abc": "https://github.com/indico/indico/issues/{1}"
    }
    with pytest.raises(ValueError):
        indico_md.to_html(source, rules)