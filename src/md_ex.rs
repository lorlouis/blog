use std::collections::BTreeMap;
use std::io::{BufRead, self};

pub fn md_to_html(s: &str) -> String {
    markdown::to_html_with_options(
        s,
        &markdown::Options {
        parse: markdown::ParseOptions {
            gfm_strikethrough_single_tilde: true,
            constructs: markdown::Constructs {
                autolink: true,
                character_escape: true,
                gfm_footnote_definition: true,
                gfm_label_start_footnote: true,
                gfm_strikethrough: true,
                gfm_table: true,
                ..Default::default()
            },
            ..Default::default()
        },
        compile: markdown::CompileOptions::gfm(),
    })
    .unwrap()
}

#[derive(Debug)]
pub enum HeaderError {
    NoValue {
        key: String,
    },
    IO(io::Error)
}

impl From<io::Error> for HeaderError {
    fn from(e: io::Error) -> Self {
        HeaderError::IO(e)
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct ExtendedMd {
    pub header: BTreeMap<String, String>,
    markdown_str: String,
}

impl ExtendedMd {

    pub fn read_header(reader: impl BufRead) -> Result<BTreeMap<String, String>, HeaderError> {
        let mut map = BTreeMap::new();
        for res in reader.lines() {
            let line = res?;
            let line_trimed = line.trim();

            if line_trimed.is_empty() {
                continue;
            }

            // end of header
            if line_trimed.chars().all(|c| c == '-') {
                if !map.is_empty() {
                    break;
                }
                else {
                    // support --- before header
                    continue;
                }
            }

            if let Some((key, rest)) = line_trimed.split_once(':') {
                let rest = rest.trim();
                if rest.is_empty() {
                    return Err(HeaderError::NoValue { key: line_trimed.to_string() })
                }
                map.insert(key.to_string(), rest.to_string());
            }
            else {
                return Err(HeaderError::NoValue { key: line_trimed.to_string() })
            }
        }
        Ok(map)
    }

    pub fn from_bufread(mut reader: impl BufRead) -> Result<Self, HeaderError> {
        let header = Self::read_header(&mut reader)?;
        let mut markdown_str = String::new();
        reader.read_to_string(&mut markdown_str)?;

        Ok(Self {
            header,
            markdown_str,
        })
    }

    pub fn to_html(&self) -> String {
        md_to_html(&self.markdown_str)
    }

}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_read_header() {
        use std::io::Cursor;
        let document = r#"---
Title: Hello world
Author: Louis: Sven

Meme: Review
---
# Actual content

More content
"#;
        let md = ExtendedMd::from_bufread(Cursor::new(document.as_bytes())).unwrap();
        assert_eq!(md, ExtendedMd {
            header: BTreeMap::from_iter(vec![
                        ("Title".to_string(), "Hello world".to_string()),
                        ("Author".to_string(), "Louis: Sven".to_string()),
                        ("Meme".to_string(), "Review".to_string()),
            ].into_iter()),
            markdown_str: r#"# Actual content

More content
"#.to_string(),
        });

    }
}
