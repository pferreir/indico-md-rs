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
        r"\bgh:(\d+)\b": "https://github.com/indico/indico/issues/{1}",
    }
    result = """<h2><a href="#test" aria-hidden="true" class="anchor" id="indico-md-test"></a>TEST</h2>
<ul>
<li><a href="https://tkt.sys/1234567" title="TKT1234567" target="_blank">TKT1234567</a>: solved</li>
<li>Still checking <a href="https://github.com/indico/indico/issues/123" title="gh:123" target="_blank">gh:123</a></li>
<li><a href="https://somewhere.else" target="_blank">gh:124</a> shouldn't be autolinked</li>
</ul>
"""
    assert indico_md.to_html(source, link_rules=rules) == result
    assert (
        indico_md.to_unstyled_html(
            "## title\n[`link`](https://example.com)\\\n`more` **text**"
        )
        == "title\n<p>link<br />\nmore text</p>\n"
    )


def test_nl2br():
    assert indico_md.to_html('hello\nworld') == '<p>hello\nworld</p>\n'
    assert indico_md.to_html('hello\nworld', nl2br=True) == '<p>hello<br />\nworld</p>\n'
    assert indico_md.to_unstyled_html('hello\nworld') == '<p>hello\nworld</p>\n'
    assert indico_md.to_unstyled_html('hello\nworld', nl2br=True) == '<p>hello<br />\nworld</p>\n'


def test_exceptions():
    source = "TEST"
    rules = {r"\bTKT(\d{7})\b": "https://tkt.sys/{1}", r"\bgh:(\d+)\b": 1234}
    with pytest.raises(TypeError):
        indico_md.to_html(source, link_rules=rules)

    rules = {
        r"\bTKT(\d{7})\b": "https://tkt.sys/{1}",
        r"(abc": "https://github.com/indico/indico/issues/{1}",
    }
    with pytest.raises(ValueError):
        indico_md.to_html(source, link_rules=rules)
    with pytest.raises(TypeError):
        indico_md.to_html(source, link_rules=[])


def test_args():
    # source is pos-only, everything else is kw-only
    with pytest.raises(TypeError):
        indico_md.to_html(source='')
    with pytest.raises(TypeError):
        indico_md.to_html('', {})
