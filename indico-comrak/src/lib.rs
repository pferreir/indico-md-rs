//! This module provides functionality for processing Markdown text with custom link rules.
//!
//! It defines the [`LinkRule`] struct for matching links using regular expressions and
//! provides the `indico_markdown_to_html` function to convert Markdown text into HTML while
//! applying the specified link rules.
//!
//! # Usage
//!
//! To use this module, create instances of [`LinkRule`] with the desired regex patterns and
//! URLs, and then call `indico_markdown_to_html` with the Markdown text and the link rules to
//! generate the HTML output.

use comrak::{
    Arena, Node, Options, create_formatter,
    html::ChildRendering,
    nodes::{ListDelimType, ListType, NodeLink, NodeValue},
    parse_document,
};
use core::fmt;
use regex_lite::Regex;
use std::fmt::{Display, Formatter, Write};

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

/// A formatter which only allows paragraphs and breaks, and ignores everything else.
/// The context's user data is a Vec we will use as a stack of lists.
fn plain_text_formatter<'a>(
    context: &mut comrak::html::Context<Vec<usize>>,
    node: &'a comrak::nodes::AstNode<'a>,
    entering: bool,
) -> Result<comrak::html::ChildRendering, std::fmt::Error> {
    match node.data().value {
        NodeValue::Code(ref nc) => {
            if entering {
                context.escape(&nc.literal)?;
            }
            Ok(ChildRendering::HTML)
        }
        NodeValue::CodeBlock(ref nc) => {
            if entering {
                context.write_str("\n")?;
                context.escape(&nc.literal)?;
                context.write_str("\n")?;
            }
            Ok(ChildRendering::HTML)
        }
        // Inline <p>...</p> and <br> are taken into account
        NodeValue::HtmlInline(ref html) => {
            let br_re = Regex::new(r"<\s*br\s*\/?>").unwrap();
            let p_open_re = Regex::new(r"<\s*p(?:\s[^>]*)?>").unwrap();

            let html = html.to_lowercase();
            if entering {
                if br_re.is_match(&html) {
                    context.write_str("<br />")?;
                } else if p_open_re.is_match(&html) {
                    context.write_str("<p>")?;
                } else if html == "</p>" {
                    context.write_str("</p>")?;
                }
            }
            Ok(ChildRendering::HTML)
        }
        // Text, paragraphs and breaks stay the same
        NodeValue::Text(..)
        | NodeValue::Paragraph
        | NodeValue::SoftBreak
        | NodeValue::LineBreak => comrak::html::format_node_default(context, node, entering),
        // Text decoration is ignored
        NodeValue::Strong
        | NodeValue::Emph
        | NodeValue::Strikethrough
        | NodeValue::Highlight
        | NodeValue::Superscript
        | NodeValue::BlockQuote => Ok(ChildRendering::HTML),
        // Lists are rendered in plain text and formatting is handled through a stack
        NodeValue::List(..) => {
            context.write_str("\n")?;
            if entering {
                context.user.push(1);
            } else {
                context.user.pop();
            }
            Ok(ChildRendering::HTML)
        }
        NodeValue::Item(lst) => {
            if entering {
                // add indentation based on stack length
                context.write_str(&" ".repeat(2 * context.user.len()))?;
                let item_spec = match lst.list_type {
                    ListType::Bullet => (lst.bullet_char as char).to_string(),
                    ListType::Ordered => {
                        format!(
                            "{}{}",
                            context.user.last().unwrap(),
                            match lst.delimiter {
                                ListDelimType::Period => ".",
                                ListDelimType::Paren => ")",
                            }
                        )
                    }
                };
                context.write_str(&item_spec)?;
                context.write_str(&" ".repeat(lst.padding - item_spec.len()))?;
            } else {
                context.write_char('\n')?;
                if let Some(v) = context.user.last_mut() {
                    *v += 1;
                };
            }
            Ok(ChildRendering::HTML)
        }
        _ => Ok(ChildRendering::HTML),
    }
}

// A formatter which adds `target="_blank"` to all links
create_formatter!(
    TargetBlankFormatter, {
        NodeValue::Link(ref nl) => |context, entering| {
            if entering {
                context.write_str(&format!("<a href=\"{}\" {}target=\"_blank\">", nl.url, if nl.title.is_empty() {
                    ""
                } else {
                    &format!("title=\"{}\" ", nl.title)
                }))?;
            } else {
                context.write_str("</a>")?;
            }
        }
    }
);

/// Manipulate the AST in order to find text nodes which match the rules, and split them
/// into the corresponding links.
fn add_links<'t>(root: &mut Node<'t>, arena: &'t Arena<'t>, link_rules: &[LinkRule]) {
    let mut to_process = Vec::new();
    let mut in_html_link = false;

    for node in root.descendants() {
        let mut n = node.data.borrow_mut();

        match &mut n.value {
            // it's a text node, so it's worth a look
            NodeValue::Text(t) => {
                let mut matches = Vec::new();

                if in_html_link {
                    // we're in a HTML link, so we shouldn't be doing any changes here
                    continue;
                }

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
            NodeValue::HtmlInline(content) => {
                // We allow raw HTML links, so we have to keep track of any open <a> tags
                if content.starts_with("<a ") {
                    in_html_link = true;
                } else if content.starts_with("</a>") {
                    in_html_link = false;
                }
            }
            _ => {}
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
pub fn indico_markdown_to_html(
    md_source: &str,
    autolink_rules: &[LinkRule],
    hardbreaks: bool,
) -> Result<String, fmt::Error> {
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
    options.render.r#unsafe = true;
    options.render.hardbreaks = hardbreaks;

    let arena = Arena::new();
    let mut root = parse_document(&arena, md_source, &options);

    add_links(&mut root, &arena, autolink_rules);

    let mut out = String::new();
    TargetBlankFormatter::format_document(root, &options, &mut out)?;

    Ok(out)
}

/// Convert markdown to plain text, which only renders paragraphs and line breaks and ignores all other rendering
pub fn indico_markdown_to_unstyled_html(
    md_source: &str,
    hardbreaks: bool,
) -> Result<String, fmt::Error> {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.tasklist = true;
    options.extension.alerts = true;
    options.extension.underline = true;
    options.extension.highlight = true;
    options.render.hardbreaks = hardbreaks;

    let arena = Arena::new();
    let root = parse_document(&arena, md_source, &options);
    let mut out = String::new();

    comrak::html::format_document_with_formatter(
        root,
        &options,
        &mut out,
        &Default::default(),
        plain_text_formatter,
        Vec::new(),
    )
    .unwrap_or_else(|_| unreachable!("writing to String cannot fail"));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{LinkRule, indico_markdown_to_html, indico_markdown_to_unstyled_html};

    #[test]
    fn test_highlight_text() {
        let md = r#"==This is important=="#;
        let html = indico_markdown_to_html(md, &[], false).unwrap();
        // should include the language class and the code content
        assert_eq!(html, "<p><mark>This is important</mark></p>\n");
    }

    #[test]
    fn test_autolink() {
        let md = r#"## TEST
 https://example.com
"#;
        let res = indico_markdown_to_html(md, &[], false).unwrap();
        assert_eq!(
            res,
            r##"<h2><a href="#test" aria-hidden="true" class="anchor" id="indico-md-test"></a>TEST</h2>
<p><a href="https://example.com" target="_blank">https://example.com</a></p>
"##
        );
    }

    #[test]
    fn test_indico_autolink() {
        let md = r#"## TEST
 * TKT1234567: solved
 * Still checking gh:123
 * [gh:124](https://somewhere.else) shouldn't be autolinked
"#;
        let res = indico_markdown_to_html(
            md,
            &[
                LinkRule::new(r"\bTKT(\d{7})\b", "https://tkt.sys/{1}").unwrap(),
                LinkRule::new(
                    r"\bgh:(\d+)\b",
                    "https://github.com/indico/indico/issues/{1}",
                )
                .unwrap(),
            ],
            false,
        )
        .unwrap();
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

        let res =
            indico_markdown_to_html("FOO", &[LinkRule::new(r"FOO", "{0}BAR").unwrap()], false)
                .unwrap();
        assert_eq!(
            res,
            "<p><a href=\"FOOBAR\" title=\"FOO\" target=\"_blank\">FOO</a></p>\n"
        );

        let res = indico_markdown_to_html(
            "FOO is FOO and BAR is BAR",
            &[
                LinkRule::new(r"(F)(O)(O)", "{1}{2}{3}BAR").unwrap(),
                LinkRule::new(r"BAR", "FOO{0}").unwrap(),
            ],
            false,
        )
        .unwrap();
        assert_eq!(
            res,
            "<p><a href=\"FOOBAR\" title=\"FOO\" target=\"_blank\">FOO</a> is <a href=\"FOOBAR\" title=\"FOO\" target=\"_blank\">FOO</a> \
and <a href=\"FOOBAR\" title=\"BAR\" target=\"_blank\">BAR</a> is <a href=\"FOOBAR\" title=\"BAR\" target=\"_blank\">BAR</a></p>\n"
        );
    }

    #[test]
    fn test_raw_html() {
        // raw HTML should be escaped when tagfilter is enabled
        let md = "<script>alert('x')</script>";
        let html = indico_markdown_to_html(md, &[], false).unwrap();
        assert_eq!(html, "&lt;script>alert('x')&lt;/script>\n");

        let md = "<div>FOO</div>";
        let html = indico_markdown_to_html(
            md,
            &[LinkRule::new(r"FOO", "https://example/{0}").unwrap()],
            false,
        )
        .unwrap();
        assert_eq!(html, "<div>FOO</div>\n");

        let md = "<a href=\"http://something.com\">FOO</a>";
        let html = indico_markdown_to_html(
            md,
            &[LinkRule::new(r"FOO", "https://example/{0}").unwrap()],
            false,
        )
        .unwrap();
        assert_eq!(html, "<p><a href=\"http://something.com\">FOO</a></p>\n");

        // inline HTML-like tags are also escaped rather than rendered
        let md = "A <b>bold</b> move";
        let html = indico_markdown_to_html(md, &[], false).unwrap();
        assert_eq!(html, "<p>A <b>bold</b> move</p>\n");
    }

    #[test]
    fn test_indico_md_to_plain() {
        let md = "[**Foo**](https://example.com)\n\n==B`ar`==<div>foo</div>";
        let html = indico_markdown_to_unstyled_html(md, false).unwrap();
        assert_eq!(html, "<p>Foo</p>\n<p>Barfoo</p>\n");

        let md = "soft\\\nvs hard break\n\nhello";
        let html = indico_markdown_to_unstyled_html(md, false).unwrap();
        assert_eq!(html, "<p>soft<br />\nvs hard break</p>\n<p>hello</p>\n");

        let md = "soft<br/>vs hard break<p>hello</p>";
        let html = indico_markdown_to_unstyled_html(md, false).unwrap();
        assert_eq!(html, "<p>soft<br />vs hard break<p>hello</p></p>\n");

        let md = "* a list\n* of\n  - nested\n* things";
        let html = indico_markdown_to_unstyled_html(md, false).unwrap();
        assert_eq!(
            html,
            "\n  * a list\n  * of\n    - nested\n\n\n  * things\n\n"
        );

        let md = "1. a list\n2. of\n    - nested\n3. ordered things";
        let html = indico_markdown_to_unstyled_html(md, false).unwrap();
        assert_eq!(
            html,
            "\n  1. a list\n  2. of\n    - nested\n\n\n  3. ordered things\n\n"
        );
    }

    #[test]
    fn test_hardbreaks() {
        // linebreaks should be converted to HTML linebreaks if enabled
        let md = "hello\nworld";
        let html = indico_markdown_to_unstyled_html(md, false).unwrap();
        assert_eq!(html, "<p>hello\nworld</p>\n");
        let html = indico_markdown_to_html(md, &[], false).unwrap();
        assert_eq!(html, "<p>hello\nworld</p>\n");
        let html = indico_markdown_to_unstyled_html(md, true).unwrap();
        assert_eq!(html, "<p>hello<br />\nworld</p>\n");
        let html = indico_markdown_to_html(md, &[], true).unwrap();
        assert_eq!(html, "<p>hello<br />\nworld</p>\n");
    }
}
