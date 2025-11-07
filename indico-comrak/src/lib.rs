//! This module provides functionality for processing Markdown text with custom link rules.
//!
//! It defines the [`LinkRule`] struct for matching links using regular expressions and
//! provides the `indico_markdown` function to convert Markdown text into HTML while
//! applying the specified link rules.
//!
//! # Usage
//!
//! To use this module, create instances of [`LinkRule`] with the desired regex patterns and
//! URLs, and then call `indico_markdown` with the Markdown text and the link rules to
//! generate the HTML output.

use comrak::{
    Arena, Node, Options, format_html_with_plugins,
    nodes::{NodeLink, NodeValue},
    options::Plugins,
    parse_document,
};
use core::fmt;
use regex_lite::Regex;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
/// Represents a rule for matching links.
///
/// The `LinkRule` struct contains a regular expression and a URL string.
/// It is used to define how links should be matched and processed within
/// the application.
///
/// # Fields
///
/// - `re`: A [`Regex`] instance that defines the pattern for matching links.
/// - `url`: A [`String`] that holds the URL associated with the link rule.
pub struct LinkRule {
    re: Regex,
    url: String,
}

#[derive(Debug)]
/// Error type that occurs when constructing link rules with invalid regular expressions.
/// Wraps the underlying [`regex_lite::Error`].
pub struct LinkRuleError(regex_lite::Error);

impl LinkRule {
    pub fn new(regex: &str, url: &str) -> Result<Self, LinkRuleError> {
        Ok(Self {
            re: Regex::new(regex).map_err(LinkRuleError)?,
            url: url.into(),
        })
    }
}

impl Display for LinkRuleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

/// Check whether any ancestor of the given node is a link
fn has_link_ancestor(node: Node<'_>) -> bool {
    if let NodeValue::Link(_) = node.data.borrow().value {
        true
    } else {
        match node.parent() {
            None => false,
            Some(n) => has_link_ancestor(n),
        }
    }
}

/// Substitute `{1},{2}...{N}` sequences in a given URL, taking into account the
/// groups which are passed
fn substitute_url(url: &str, groups: &[Option<String>]) -> String {
    let mut res: String = url.into();
    for (n, group) in groups
        .iter()
        .enumerate()
        .filter_map(|(n, g)| g.as_ref().map(|g| (n, g)))
    {
        let marker = &format!("{{{}}}", n);
        res = res.replace(marker, group);
    }
    res
}

/// Manipulate the AST in order to find text nodes which match the rules, and split them
/// into the corresponding links.
fn add_links<'t>(root: &mut Node<'t>, arena: &'t Arena<'t>, link_rules: &[LinkRule]) {
    let mut to_process = Vec::new();

    for node in root.descendants() {
        let mut n = node.data.borrow_mut();

        // it's a text node, so it's worth a look
        if let NodeValue::Text(t) = &mut n.value {
            let mut matches = Vec::new();

            // check if any of the rules match
            for LinkRule { re, url } in link_rules {
                // go over the captured parts of the text
                for capture in re.captures_iter(t) {
                    let groups: Vec<_> = capture
                        .iter()
                        .map(|c| c.map(|m| m.as_str().to_string()))
                        .collect();
                    let start = capture
                        .iter()
                        .filter_map(|c| c.map(|c| c.start()))
                        .min()
                        .unwrap();
                    let end = capture
                        .iter()
                        .filter_map(|c| c.map(|c| c.end()))
                        .max()
                        .unwrap();

                    matches.push(((start, end), url, groups))
                }
            }
            if !matches.is_empty() {
                // one line per node
                to_process.push((node, t.to_string(), matches));
            }
        }
    }

    for (node, text, matches) in to_process {
        // Exclude nodes whose ancestor is a link
        if has_link_ancestor(node) {
            continue;
        }

        let parent = node.parent().unwrap();
        node.detach();

        let mut prev_end = 0;

        // let's check each match one by one
        for ((start, end), url, capture_groups) in &matches {
            parent.append(
                arena.alloc(NodeValue::Text(text[prev_end..*start].to_string().into()).into()),
            );

            let link = arena.alloc(
                NodeValue::Link(Box::new(NodeLink {
                    url: substitute_url(url, capture_groups),
                    title: text[*start..*end].into(),
                }))
                .into(),
            );
            link.append(arena.alloc(NodeValue::Text(text[*start..*end].to_string().into()).into()));

            parent.append(link);
            prev_end = *end;
        }

        let last_end = matches.last().unwrap().0.1;

        if last_end != text.len() {
            parent.append(arena.alloc(NodeValue::Text(text[last_end..].to_string().into()).into()));
        }
    }
}

/// Main function in the module, which takes a markdown string and a list of rules, and returns
/// the resulting HTML
pub fn indico_markdown(md_source: &str, autolink_rules: &[LinkRule]) -> Result<String, fmt::Error> {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.header_ids = Some("indico-md-".into());
    options.extension.tagfilter = true;
    options.extension.table = true;
    options.extension.tasklist = true;
    options.extension.alerts = true;
    options.extension.autolink = true;
    options.extension.math_code = true;
    options.extension.math_dollars = true;
    options.extension.underline = true;
    options.extension.highlight = true;

    let arena = Arena::new();
    let mut root = parse_document(&arena, md_source, &options);

    add_links(&mut root, &arena, autolink_rules);

    let mut out = String::new();
    format_html_with_plugins(root, &options, &mut out, &Plugins::default())?;

    Ok(out)
}

#[cfg(test)]
mod tests {
    use crate::LinkRule;

    use super::indico_markdown;

    #[test]
    fn test_highlight_text() {
        let md = r#"==This is important=="#;
        let html = indico_markdown(md, &[]).unwrap();
        // should include the language class and the code content
        assert_eq!(html, "<p><mark>This is important</mark></p>\n");
    }

    #[test]
    fn test_indico_autolink() {
        let md = r#"## TEST
 * TKT1234567: solved
 * Still checking gh:123
 * [gh:124](https://somewhere.else) shouldn't be autolinked
"#;
        let res = indico_markdown(
            md,
            &[
                LinkRule::new(r"\bTKT(\d{7})\b", "https://tkt.sys/{1}").unwrap(),
                LinkRule::new(
                    r"\bgh:(\d+)\b",
                    "https://github.com/indico/indico/issues/{1}",
                )
                .unwrap(),
            ],
        )
        .unwrap();
        assert_eq!(
            res,
            r##"<h2><a href="#test" aria-hidden="true" class="anchor" id="indico-md-test"></a>TEST</h2>
<ul>
<li><a href="https://tkt.sys/1234567" title="TKT1234567">TKT1234567</a>: solved</li>
<li>Still checking <a href="https://github.com/indico/indico/issues/123" title="gh:123">gh:123</a></li>
<li><a href="https://somewhere.else">gh:124</a> shouldn't be autolinked</li>
</ul>
"##
        );

        let res = indico_markdown("FOO", &[LinkRule::new(r"FOO", "{0}BAR").unwrap()]).unwrap();
        assert_eq!(res, "<p><a href=\"FOOBAR\" title=\"FOO\">FOO</a></p>\n");

        let res = indico_markdown(
            "FOO is FOO and BAR is BAR",
            &[
                LinkRule::new(r"(F)(O)(O)", "{1}{2}{3}BAR").unwrap(),
                LinkRule::new(r"BAR", "FOO{0}").unwrap(),
            ],
        )
        .unwrap();
        assert_eq!(
            res,
            "<p><a href=\"FOOBAR\" title=\"FOO\">FOO</a> is <a href=\"FOOBAR\" title=\"FOO\">FOO</a> \
and <a href=\"FOOBAR\" title=\"BAR\">BAR</a> is <a href=\"FOOBAR\" title=\"BAR\">BAR</a></p>\n"
        );
    }
}
