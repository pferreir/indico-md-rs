//! A module for converting Markdown text to HTML with custom link rules.
//!
//! This module provides functionality to transform Markdown strings into HTML while applying
//! user-defined link rules for automatic URL transformations. It utilizes the Indico library's
//! capabilities to handle Markdown parsing and link generation.
//!
//! The primary function exposed by this module is `to_html`, which takes a Markdown string and
//! a set of link rules defined as regular expressions and their corresponding URL replacements.
//! It returns the resulting HTML as a string, wrapped in a PyResult to handle potential errors
//! during the conversion process.
use indico_comrak::{LinkRule, indico_markdown_to_html, indico_markdown_to_unstyled_html};
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
};
use std::collections::HashMap;

/// Converts Markdown text to HTML with custom link rules.
///
/// This function takes a Markdown string and a set of link rules, converts the Markdown to HTML
/// while applying the specified Indico auto-link rules for URL transformations.
///
/// # Arguments
///
/// * `md_source` - A string slice containing the Markdown text to convert
/// * `link_rules` - A HashMap containing pairs of regular expression patterns (as strings) and
///                  their corresponding URL replacements
///
/// # Returns
///
/// * [`PyResult<String>`] - The resulting HTML string wrapped in a PyResult
///
/// # Errors
///
/// Returns a [`PyValueError`] if any of the regular expressions in the link rules are invalid
///
/// # Example
///
/// ```python
/// import indico_md
///
/// # Convert #1234 to a GitHub issue link
/// md_text = "See issue #1234 for details"
/// link_rules = {"#([0-9]+)": "https://github.com/org/repo/issues/$1"}
/// html = indico_md.to_html(md_text, link_rules)
/// # Output: '<p>See issue <a href="https://github.com/org/repo/issues/1234">#1234</a> for details</p>'
/// ```
#[pyfunction]
fn to_html(md_source: &str, link_rules: HashMap<String, String>) -> PyResult<String> {
    let rules: Vec<_> = link_rules
        .iter()
        .map(|(re, url)| LinkRule::new(re, url))
        .collect::<Result<_, _>>()
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    indico_markdown_to_html(md_source, &rules).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pyfunction]
fn to_unstyled_html(md_source: &str) -> PyResult<String> {
    indico_markdown_to_unstyled_html(md_source).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pymodule]
fn indico_md(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(to_html, m)?)?;
    m.add_function(wrap_pyfunction!(to_unstyled_html, m)?)?;
    Ok(())
}
