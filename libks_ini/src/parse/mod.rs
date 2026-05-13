mod line;
use line::{Line, next_line};

mod trim;
use trim::{trimmed_range_start, trimmed_range_end};

use crate::item::*;
use crate::span::Span;
use crate::whitespace::{Padding2, Padding4};

pub struct Parser<'a> {
    source: &'a str,
    next_offset: Option<usize>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            next_offset: Some(0),
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        let source = self.source;
        let offset = self.next_offset?;
        
        let Line {
            full,
            trimmed,
            next_offset,
            line_ending,
        } = next_line(source, offset);
        
        let line_trimmed = &source[trimmed.clone()];
        let ws_before = Span::from(full.start..trimmed.start);
        let ws_after = Span::from(trimmed.end..full.end);

        let item: Item = match line_trimmed.chars().next() {
            // Section key
            Some('[') => match line_trimmed.chars().last().unwrap() {
                ']' => Item::Section(SectionItem {
                    key: Span::from(trimmed.start + 1..trimmed.end - 1),
                    padding: Padding2(ws_before, ws_after),
                    line_ending,
                }),
                _ => Item::Error(ErrorItem {
                    line: Span::from(full),
                    line_ending,
                }),
            },
            // Comment
            Some(';') => Item::Comment(CommentItem {
                comment: Span::from(trimmed.start + 1..trimmed.end),
                padding: Padding2(ws_before, ws_after),
                line_ending,
            }),
            // Property
            Some(_) => match memchr::memchr(b'=', line_trimmed.as_bytes()) {
                Some(mut eq_index) => {
                    eq_index += trimmed.start;
                    
                    let key_untrimmed = &source[trimmed.start..eq_index];
                    let key_end = trimmed.start + trimmed_range_end(key_untrimmed);
                    let key = Span::from(trimmed.start..key_end);
                    let before_eq = Span::from(key_end..eq_index);
                    
                    let value_untrimmed = &source[eq_index + 1..trimmed.end];
                    let value_start = eq_index + 1 + trimmed_range_start(value_untrimmed);
                    let value = Span::from(value_start..trimmed.end);
                    let after_eq = Span::from(eq_index + 1..value_start);
                    
                    Item::Property(PropertyItem {
                        key,
                        value,
                        padding: Padding4(ws_before, before_eq, after_eq, ws_after),
                        line_ending,
                    })
                },
                None => Item::Error(ErrorItem {
                    line: Span::from(full),
                    line_ending,
                }),
            },
            // Blank
            None => Item::Blank(BlankItem {
                line: Span::from(full),
                line_ending,
            }),
        };

        self.next_offset = next_offset;
        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::whitespace::*;
    
    macro_rules! padding {
        ($p1:literal, $p2:literal) => {
            Padding2(
                const_str::repeat!(" ", $p1).into(),
                const_str::repeat!(" ", $p2).into(),
            )
        };
        ($p1:literal, $p2:literal, $p3:literal, $p4:literal) => {
            Padding4(
                const_str::repeat!(" ", $p1).into(),
                const_str::repeat!(" ", $p2).into(),
                const_str::repeat!(" ", $p3).into(),
                const_str::repeat!(" ", $p4).into(),
            )
        };
    }

    macro_rules! section {
        ($key:literal, pad=$padding:expr, end=$ending:expr) => {
            Item::Section(SectionItem {
                key: $key.into(),
                padding: $padding,
                line_ending: $ending,
            })
        };
        ($key:literal, end=$ending:expr) => {
            section!($key, pad=Padding2::default(), end=$ending)
        };
        ($key:literal, pad=$padding:expr) => {
            section!($key, pad=$padding, end=LineEnding::default())
        };
        ($key:literal) => {
            section!($key, pad=Padding2::default(), end=LineEnding::default())
        };
    }

    macro_rules! prop {
        ($key:literal => $value:literal, pad=$padding:expr, end=$ending:expr) => {
            Item::Property(PropertyItem {
                key: $key.into(),
                value: $value.into(),
                padding: $padding,
                line_ending: $ending,
            })
        };
        ($key:literal => $value:literal, end=$ending:expr) => {
            prop!($key => $value, pad=Padding4::default(), end=$ending)
        };
        ($key:literal => $value:literal, pad=$padding:expr) => {
            prop!($key => $value, pad=$padding, end=LineEnding::default())
        };
        ($key:literal => $value:literal) => {
            prop!($key => $value, pad=Padding4::default(), end=LineEnding::default())
        };
    }

    macro_rules! comment {
        ($comment:literal, pad=$padding:expr, end=$ending:expr) => {
            Item::Comment(CommentItem {
                comment: $comment.into(),
                padding: $padding,
                line_ending: $ending,
            })
        };
        ($comment:literal, end=$ending:expr) => {
            comment!($comment, pad=Padding2::default(), end=$ending)
        };
        ($comment:literal, pad=$padding:expr) => {
            comment!($comment, pad=$padding, end=LineEnding::default())
        };
        ($comment:literal) => {
            comment!($comment, pad=Padding2::default(), end=LineEnding::default())
        };
    }

    macro_rules! blank {
        ($line:literal, end=$ending:expr) => {
            Item::Blank(BlankItem {
                line: $line.into(),
                line_ending: $ending,
            })
        };
        ($line:literal) => {
            blank!($line, end=LineEnding::default())
        };
        () => {
            blank!("")
        };
    }

    macro_rules! error {
        ($line:literal, end=$ending:expr) => {
            Item::Error(ErrorItem {
                line: $line.into(),
                line_ending: $ending,
            })
        };
        ($line:literal) => {
            error!($line, end=LineEnding::default())
        };
    }

    #[test]
    fn parser_works() {
        let source = "\
;Hello
[World]
Name=The Machine
Author=Nifflas

[x1000y1000]
ShiftVisible(A)=False
ShiftEffect(A)=False
ShiftSound(A)=None";
        let parser = Parser::new(source);
        let actual: Vec<_> = parser
            .map(|item| item.into_owned(source))
            .collect();
        let expected = [
            comment!("Hello"),
            section!("World"),
            prop!("Name" => "The Machine"),
            prop!("Author" => "Nifflas"),
            blank!(),
            section!("x1000y1000"),
            prop!("ShiftVisible(A)" => "False"),
            prop!("ShiftEffect(A)" => "False"),
            prop!("ShiftSound(A)" => "None", end=LineEnding::None),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn parser_recognizes_errors() {
        let source = "\
[World] invalid
Name

[x1000y1000
=False";
        let parser = Parser::new(source);
        let actual: Vec<_> = parser
            .map(|item| item.into_owned(source))
            .collect();
        let expected = [
            error!("[World] invalid"),
            error!("Name"),
            blank!(),
            error!("[x1000y1000"),
            prop!("" => "False", end=LineEnding::None),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn parser_handles_whitespace_correctly() {
        let source = "  ;  Hello  
  [World]
  Name=The Machine
Author  =Nifflas
     
[x1000y1000]  
ShiftVisible(A)=  False
ShiftEffect(A)=False  
  ShiftSound(A)  =  None  ";
        let parser = Parser::new(source);
        let actual: Vec<_> = parser
            .map(|item| item.into_owned(source))
            .collect();
        let expected = [
            comment!("  Hello", pad=padding!(2, 2)),
            section!("World", pad=padding!(2, 0)),
            prop!("Name" => "The Machine", pad=padding!(2, 0, 0, 0)),
            prop!("Author" => "Nifflas", pad=padding!(0, 2, 0, 0)),
            blank!("     "),
            section!("x1000y1000", pad=padding!(0, 2)),
            prop!("ShiftVisible(A)" => "False", pad=padding!(0, 0, 2, 0)),
            prop!("ShiftEffect(A)" => "False", pad=padding!(0, 0, 0, 2)),
            prop!("ShiftSound(A)" => "None", pad=padding!(2, 2, 2, 2), end=LineEnding::None),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn parser_handles_newlines_correctly() {
        let source = "\
;Hello\r\
[World]\n\
Name=The Machine\r\n\
Author=Nifflas\r     \n\
[x1000y1000]\r\n\
ShiftVisible(A)=False\r\
ShiftEffect(A)=False\n\
ShiftSound(A)=None\r\n\
";
        let parser = Parser::new(source);
        let actual: Vec<_> = parser
            .map(|item| item.into_owned(source))
            .collect();
        let expected = [
            comment!("Hello", end=LineEnding::Cr),
            section!("World", end=LineEnding::Lf),
            prop!("Name" => "The Machine", end=LineEnding::CrLf),
            prop!("Author" => "Nifflas", end=LineEnding::Cr),
            blank!("     "),
            section!("x1000y1000", end=LineEnding::CrLf),
            prop!("ShiftVisible(A)" => "False", end=LineEnding::Cr),
            prop!("ShiftEffect(A)" => "False", end=LineEnding::Lf),
            prop!("ShiftSound(A)" => "None", end=LineEnding::CrLf),
            blank!("", end=LineEnding::None),
        ];
        assert_eq!(actual, expected);
     }
}
