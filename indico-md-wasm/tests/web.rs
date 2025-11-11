//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use indico_md_wasm::{to_html, to_unstyled_html};
use js_sys::{Array, RegExp};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn function_test() {
    let md = r#"## TEST
 * TKT1234567: solved
 * Still checking gh:123
 * [gh:124](https://somewhere.else) shouldn't be autolinked
"#;
    let rules = Array::new();
    rules.push(&Array::of2(
        &RegExp::new(r"\bTKT(\d{7})\b", ""),
        &JsValue::from("https://tkt.sys/{1}"),
    ));
    rules.push(&Array::of2(
        &RegExp::new(r"\bgh:(\d+)\b", ""),
        &JsValue::from("https://github.com/indico/indico/issues/{1}"),
    ));

    let res = to_html(md, &rules.into(), false).unwrap();

    assert_eq!(
        res,
        r##"<h2><a href="#test" aria-hidden="true" class="anchor" id="indico-md-test"></a>TEST</h2>
<ul>
<li><a href="https://tkt.sys/1234567" title="TKT1234567" target="_blank">TKT1234567</a>: solved</li>
<li>Still checking <a href="https://github.com/indico/indico/issues/123" title="gh:123" target="_blank">gh:123</a></li>
<li><a href="https://somewhere.else" target="_blank">gh:124</a> shouldn't be autolinked</li>
</ul>
"##
    );

    assert_eq!(
        to_unstyled_html(
            "## title\n[`link`](https://example.com)\\\n`more` **text**",
            false
        )
        .unwrap(),
        "title\n<p>link<br />\nmore text</p>\n"
    )
}

#[wasm_bindgen_test]
fn nl2br_test() {
    assert_eq!(
        to_html("hello\nworld", &Array::new(), false),
        Ok("<p>hello\nworld</p>\n".into())
    );
    assert_eq!(
        to_unstyled_html("hello\nworld", false),
        Ok("<p>hello\nworld</p>\n".into())
    );
    assert_eq!(
        to_html("hello\nworld", &Array::new(), true),
        Ok("<p>hello<br />\nworld</p>\n".into())
    );
    assert_eq!(
        to_unstyled_html("hello\nworld", true),
        Ok("<p>hello<br />\nworld</p>\n".into())
    );
}

#[wasm_bindgen_test]
fn interface_test() {
    assert_eq!(to_html("", &Array::new(), false), Ok("".into()));
    assert_eq!(to_html("", &Array::new(), true), Ok("".into()));

    let rules = Array::new();
    rules.push(&Array::of2(
        &RegExp::new(r"/a/", ""),
        // URL cannot be a bool, so this should fail
        &JsValue::from_bool(true),
    ));
    let res = to_html("foo", &rules, false);
    assert!(res.is_err());
    assert!(
        res.err()
            .unwrap()
            .as_string()
            .expect("Error is not a string")
            .contains("not a valid string")
    )
}
