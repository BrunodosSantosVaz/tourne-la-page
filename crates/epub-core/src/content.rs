use ego_tree::NodeRef;
use scraper::{Html, Node, Selector};

use crate::Chapter;

const MAX_HTML_DEPTH: usize = 256;

pub(crate) fn parse_chapter(source: &str, fallback_title: &str) -> Chapter {
    let document = Html::parse_document(source);
    let title = find_title(&document).unwrap_or_else(|| fallback_title.to_owned());
    let mut text = String::new();
    for child in document.root_element().children() {
        walk(child, 0, &mut text);
    }
    Chapter {
        title,
        text: normalize_lines(&text),
    }
}

fn find_title(document: &Html) -> Option<String> {
    for selector in ["body h1", "body h2", "body h3"] {
        let selector = Selector::parse(selector).ok()?;
        if let Some(title) = document
            .select(&selector)
            .map(|element| element.text().collect::<String>().trim().to_owned())
            .find(|title| !title.is_empty())
        {
            return Some(title);
        }
    }
    None
}

fn walk(node: NodeRef<'_, Node>, depth: usize, output: &mut String) {
    if depth > MAX_HTML_DEPTH {
        return;
    }
    let is_block = node
        .value()
        .as_element()
        .is_some_and(|element| is_block(element.name()));
    if is_block {
        append_newline(output);
    }
    if let Some(text) = node.value().as_text() {
        append_text(output, &text.text);
    } else if let Some(element) = node.value().as_element() {
        if !is_ignored(element.name()) {
            for child in node.children() {
                walk(child, depth + 1, output);
            }
        }
    } else {
        for child in node.children() {
            walk(child, depth + 1, output);
        }
    }
    if is_block {
        append_newline(output);
    }
}

fn append_text(output: &mut String, text: &str) {
    output.push_str(text);
}

fn append_newline(output: &mut String) {
    if !output.is_empty() && !output.ends_with("\n\n") {
        output.push_str("\n\n");
    }
}

fn normalize_lines(text: &str) -> String {
    text.split("\n\n")
        .map(|paragraph| paragraph.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|paragraph| !paragraph.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn is_ignored(name: &str) -> bool {
    matches!(
        name,
        "head" | "script" | "style" | "svg" | "iframe" | "object" | "template"
    )
}

fn is_block(name: &str) -> bool {
    matches!(
        name,
        "address"
            | "article"
            | "blockquote"
            | "br"
            | "dd"
            | "div"
            | "dl"
            | "dt"
            | "figcaption"
            | "figure"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "hr"
            | "li"
            | "ol"
            | "p"
            | "pre"
            | "section"
            | "table"
            | "td"
            | "th"
            | "tr"
            | "ul"
    )
}
