#![feature(proc_macro_quote)]
#![feature(if_let_guard)]
use proc_macro::*;

use std::iter::Peekable;

fn parse_closure(items: &Group, out_string: &mut String) {
    let mut is_move = false;
    let mut group = items.stream().into_iter().peekable();

    if let Some(TokenTree::Group(g)) = group.peek() {
        if let Delimiter::Bracket = g.delimiter() {
            if g.to_string() == "[move]" {
                is_move = true;
                // consume [move]
                group.next();
            }
        }
    }

    if is_move {
        out_string.push_str("Node::Fn(Box::new(move || {");
    } else {
        out_string.push_str("Node::Fn(Box::new(|| {");
    }

    for token in group {
        out_string.push_str(&token.to_string());
    }

    out_string.push_str("} )),");
}

fn parse_tag(stream: &mut Peekable<impl Iterator<Item=TokenTree>>, out_string: &mut String) {
    let mut last_token_was_punct=true;
    out_string.push('<');
    while let Some(token) = stream.peek() {
        match token {
            TokenTree::Punct(p) => {
                last_token_was_punct=true;
                out_string.push_str(&p.to_string());
                let c = p.as_char();
                if c == '>'  {
                    stream.next();
                    break
                }
            },
            TokenTree::Group(g) if let Delimiter::Brace = g.delimiter() => {
                out_string.push_str("\"#),");
                parse_closure(g, out_string);
                out_string.push_str("Node::Str(r#\"");
            },
            _ => {
                if last_token_was_punct {
                out_string.push_str(&format!("{token}"));
                } else {
                    out_string.push_str(&format!(" {token}"));
                }
                last_token_was_punct = false;
            }
        }
        stream.next();
    }
}

fn parse_sequence(stream: &mut Peekable<impl Iterator<Item=TokenTree>>, out_string: &mut String) {

    out_string.push_str("Node::List(vec![");

    let mut local_string = String::new();

    while let Some(token) = stream.next() {
        match token {
            TokenTree::Group(g) if let Delimiter::Brace = g.delimiter() => {
                if !local_string.is_empty() {
                    local_string.push_str("\"#),");
                    out_string.push_str(&local_string);
                    local_string.clear();
                }
                parse_closure(&g, out_string);
            },
            TokenTree::Literal(l) if is_string_literal(&l) => {
                if local_string.is_empty() {
                    local_string.push_str("Node::Str(r#\"");
                }
                let string = l.to_string();
                local_string.push_str(string.trim_matches(|c| c == '"'));
            },
            TokenTree::Punct(p) if p.as_char() == '<' => {
                if local_string.is_empty() {
                    local_string.push_str("Node::Str(r#\"");
                }
                parse_tag(stream, &mut local_string);
            },
            _ => {
                if local_string.is_empty() {
                    local_string.push_str("Node::Str(r#\"");
                }
                local_string.push_str(&token.to_string());
            }
        }
    }

    if !local_string.is_empty() {
        local_string.push_str("\"#),");
        out_string.push_str(&local_string);
    }

    out_string.push_str("]),");
}

/// shamelessly stolen from
/// <https://github.com/bodil/typed-html/blob/e18d328951b6b9216976d180f3dea2e6600a3982/macros/src/html.rs#L140>
fn is_string_literal(literal: &Literal) -> bool {
    // This is the worst API
    literal.to_string().starts_with('"')
}

#[proc_macro]
pub fn html(items: TokenStream) -> TokenStream {
    let mut out_string = r#"{
        use html_template_core::Node;
        Node::List(vec![
    "#.to_string();
    let mut stream = items.into_iter().peekable();
    while let Some(token) = stream.peek() {
        match token {
            TokenTree::Group(ref g) if let Delimiter::Brace = g.delimiter() => {
                parse_closure(g, &mut out_string);
                stream.next();
            },
            _ => parse_sequence(&mut stream, &mut out_string)
        }
    }
    out_string.push_str(r#"])}"#);
    out_string.parse().unwrap()
}
