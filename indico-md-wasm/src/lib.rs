use indico_comrak::{
    LinkRule, indico_markdown_to_html as _indico_md_to_html,
    indico_markdown_to_unstyled_html as _indico_md_to_unstyled_html,
};
use js_sys::Array;
use wasm_bindgen::prelude::*;

/// Converts markdown text to HTML while applying custom link rules
///
/// This function takes markdown text and an array of link rules from JavaScript,
/// processes them according to Indico's markdown rules, and returns the resulting HTML.
///
/// # Arguments
///
/// * `md_source` - A string slice containing the markdown text to process
/// * `js_rules` - A JavaScript array containing pairs of RegExp and URL pattern strings
///
/// # Returns
///
/// * `Result<String, JsValue>` - The processed HTML string on success, or a JsValue error on failure
///
/// # Errors
///
/// Returns a JsValue error if:
/// * The URL pattern is not a valid string
/// * The regular expression is not a valid string
/// * The link rule creation fails
///
/// # Example (JavaScript)
///
/// ```javascript
/// const rules = [
///   [/^#(\d+)$/, 'https://example.com/issues/$1'],
///   [/^@(\w+)$/, 'https://example.com/users/$1']
/// ];
/// const html = indicoMarkdown("See #123 and @user", rules);
/// ```
#[wasm_bindgen(js_name = toHtml)]
pub fn to_html(md_source: &str, js_rules: &Array) -> Result<String, JsValue> {
    let mut rules = Vec::new();

    for res in js_rules.values() {
        let array: js_sys::Array = res?.into();
        let vec: Vec<_> = array.to_vec();
        let re: js_sys::RegExp = vec[0].clone().into();
        let url_pattern = vec[1]
            .as_string()
            .ok_or(JsValue::from_str("URL pattern is not a valid string"))?;

        rules.push(
            LinkRule::new(
                &re.source().as_string().ok_or(JsValue::from_str(
                    "Regular expression is not a valid string",
                ))?,
                &url_pattern,
            )
            .map_err(|e| e.to_string())?,
        );
    }
    _indico_md_to_html(md_source, &rules).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen(js_name = toUnstyledHtml)]
pub fn to_unstyled_html(md_source: &str) -> Result<String, JsValue> {
    _indico_md_to_unstyled_html(md_source).map_err(|e| JsValue::from_str(&e.to_string()))
}
