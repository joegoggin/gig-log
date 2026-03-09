use std::iter::Peekable;
use std::str::Chars;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use serde_json::Value;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BodyPreviewFormat {
    Json,
    Text,
}

#[derive(Debug, Clone)]
pub struct BodyPreview {
    pub lines: Vec<Line<'static>>,
    pub format: BodyPreviewFormat,
}

pub fn build(body: &str) -> BodyPreview {
    let trimmed = body.trim();

    if trimmed.is_empty() {
        return BodyPreview {
            lines: vec![],
            format: BodyPreviewFormat::Text,
        };
    }

    if let Ok(value) = serde_json::from_str::<Value>(trimmed)
        && let Ok(pretty) = serde_json::to_string_pretty(&value)
    {
        return BodyPreview {
            lines: highlight_json(&pretty),
            format: BodyPreviewFormat::Json,
        };
    }

    BodyPreview {
        lines: plain_text_lines(body),
        format: BodyPreviewFormat::Text,
    }
}

fn plain_text_lines(body: &str) -> Vec<Line<'static>> {
    body.lines()
        .map(|line| Line::from(line.to_string()))
        .collect()
}

fn highlight_json(pretty_json: &str) -> Vec<Line<'static>> {
    pretty_json.lines().map(highlight_json_line).collect()
}

fn highlight_json_line(line: &str) -> Line<'static> {
    let mut chars = line.chars().peekable();
    let mut spans = vec![];

    while let Some(ch) = chars.peek().copied() {
        if ch.is_whitespace() {
            let mut whitespace = String::new();

            while let Some(current) = chars.peek().copied() {
                if current.is_whitespace() {
                    whitespace.push(current);
                    chars.next();
                } else {
                    break;
                }
            }

            spans.push(Span::raw(whitespace));
            continue;
        }

        if ch == '"' {
            let token = consume_string(&mut chars);

            let mut lookahead = chars.clone();
            while let Some(current) = lookahead.peek().copied() {
                if current.is_whitespace() {
                    lookahead.next();
                } else {
                    break;
                }
            }

            let style = if matches!(lookahead.peek(), Some(':')) {
                key_style()
            } else {
                string_style()
            };

            spans.push(Span::styled(token, style));
            continue;
        }

        if ch == '-' || ch.is_ascii_digit() {
            let token = consume_number(&mut chars);
            spans.push(Span::styled(token, number_style()));
            continue;
        }

        if ch == 't' && try_consume_literal(&mut chars, "true") {
            spans.push(Span::styled("true", boolean_style()));
            continue;
        }

        if ch == 'f' && try_consume_literal(&mut chars, "false") {
            spans.push(Span::styled("false", boolean_style()));
            continue;
        }

        if ch == 'n' && try_consume_literal(&mut chars, "null") {
            spans.push(Span::styled("null", null_style()));
            continue;
        }

        if matches!(ch, '{' | '}' | '[' | ']' | ':' | ',') {
            chars.next();
            spans.push(Span::styled(ch.to_string(), punctuation_style()));
            continue;
        }

        let mut token = String::new();

        while let Some(current) = chars.peek().copied() {
            if current.is_whitespace() || matches!(current, '{' | '}' | '[' | ']' | ':' | ',') {
                break;
            }

            token.push(current);
            chars.next();
        }

        spans.push(Span::raw(token));
    }

    Line::from(spans)
}

fn consume_string(chars: &mut Peekable<Chars<'_>>) -> String {
    let mut token = String::new();
    let mut escaped = false;

    if let Some(first) = chars.next() {
        token.push(first);
    }

    for current in chars.by_ref() {
        token.push(current);

        if escaped {
            escaped = false;
            continue;
        }

        if current == '\\' {
            escaped = true;
            continue;
        }

        if current == '"' {
            break;
        }
    }

    token
}

fn consume_number(chars: &mut Peekable<Chars<'_>>) -> String {
    let mut token = String::new();

    if let Some(first) = chars.next() {
        token.push(first);
    }

    while let Some(current) = chars.peek().copied() {
        if current.is_ascii_digit() || matches!(current, '.' | 'e' | 'E' | '+' | '-') {
            token.push(current);
            chars.next();
        } else {
            break;
        }
    }

    token
}

fn try_consume_literal(chars: &mut Peekable<Chars<'_>>, literal: &str) -> bool {
    let mut probe = chars.clone();

    for expected in literal.chars() {
        match probe.next() {
            Some(current) if current == expected => {}
            _ => return false,
        }
    }

    for _ in literal.chars() {
        chars.next();
    }

    true
}

fn punctuation_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

fn key_style() -> Style {
    Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

fn string_style() -> Style {
    Style::default().fg(Color::Green)
}

fn number_style() -> Style {
    Style::default().fg(Color::Yellow)
}

fn boolean_style() -> Style {
    Style::default().fg(Color::Magenta)
}

fn null_style() -> Style {
    Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::ITALIC)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>()
    }

    #[test]
    fn valid_json_is_pretty_printed_and_highlighted() {
        let preview = build("{\"name\":\"Gig Log\",\"active\":true}");

        assert_eq!(preview.format, BodyPreviewFormat::Json);
        assert_eq!(line_text(&preview.lines[0]), "{");
        assert_eq!(line_text(&preview.lines[1]), "  \"active\": true,");
        assert_eq!(line_text(&preview.lines[2]), "  \"name\": \"Gig Log\"");
        assert_eq!(line_text(&preview.lines[3]), "}");

        let key_span = preview.lines[1]
            .spans
            .iter()
            .find(|span| span.content == "\"active\"")
            .expect("key span should be present");
        assert_eq!(key_span.style.fg, Some(Color::Cyan));

        let bool_span = preview.lines[1]
            .spans
            .iter()
            .find(|span| span.content == "true")
            .expect("boolean span should be present");
        assert_eq!(bool_span.style.fg, Some(Color::Magenta));
    }

    #[test]
    fn invalid_json_falls_back_to_plain_text() {
        let preview = build("{\"name\":\"Gig Log\"");

        assert_eq!(preview.format, BodyPreviewFormat::Text);
        assert_eq!(preview.lines.len(), 1);
        assert_eq!(line_text(&preview.lines[0]), "{\"name\":\"Gig Log\"");
    }

    #[test]
    fn empty_body_has_no_preview_lines() {
        let preview = build("  \n  ");

        assert_eq!(preview.format, BodyPreviewFormat::Text);
        assert!(preview.lines.is_empty());
    }
}
