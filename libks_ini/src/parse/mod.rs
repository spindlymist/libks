mod line;
use line::{Line, next_line};

mod trim;
use trim::{trimmed_range_start, trimmed_range_end};

mod item;
use item::Item;

mod whitespace;
use whitespace::{Padding2, Padding4};

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
        let next_offset = self.next_offset?;
        
        let Line {
            full,
            trimmed,
            ws_before,
            ws_after,
            next_offset,
            line_ending,
        } = next_line(source, next_offset);
        
        let source_trimmed = &source[trimmed.clone()];

        let item = match source_trimmed.chars().next() {
            // Section key
            Some('[') => match source_trimmed.chars().last().unwrap() {
                ']' => Item::Section {
                    key: trimmed.start + 1..trimmed.end - 1,
                    padding: Padding2(ws_before, ws_after),
                    line_ending,
                },
                _ => Item::Error {
                    line: full,
                    line_ending,
                }
            },
            // Comment
            Some(';') => Item::Comment {
                comment: trimmed.start + 1..trimmed.end,
                padding: Padding2(ws_before, ws_after),
                line_ending,
            },
            // Property
            Some(_) => match memchr::memchr(b'=', source_trimmed.as_bytes()) {
                Some(mut eq_index) => {
                    eq_index += trimmed.start;
                    let (key, before_eq) = {
                        let untrimmed = &source[trimmed.start..eq_index];
                        let end_trimmed = trimmed.start + trimmed_range_end(untrimmed);
                        (trimmed.start..end_trimmed, end_trimmed..eq_index)
                    };
                    let (value, after_eq) = {
                        let untrimmed = &source[eq_index + 1..trimmed.end];
                        let start_trimmed = eq_index + 1 + trimmed_range_start(untrimmed);
                        (start_trimmed..trimmed.end, eq_index + 1..start_trimmed)
                    };
                    Item::Property {
                        key,
                        value,
                        padding: Padding4(ws_before, before_eq, after_eq, ws_after),
                        line_ending,
                    }
                },
                None => Item::Error {
                    line: full,
                    line_ending,
                },
            },
            // Blank
            None => Item::Blank {
                line: full,
                line_ending,
            },
        };

        self.next_offset = next_offset;
        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::{*, item::*, whitespace::*};
    
    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct OwnedPadding2(String, String);
    
    impl OwnedPadding2 {
        pub fn from_borrowed(borrowed: Padding2, source: &str) -> OwnedPadding2 {
            OwnedPadding2(
                source[borrowed.0].to_owned(),
                source[borrowed.1].to_owned(),
            )
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct OwnedPadding4(String, String, String, String);
    
    impl OwnedPadding4 {
        pub fn from_borrowed(borrowed: Padding4, source: &str) -> OwnedPadding4 {
            OwnedPadding4(
                source[borrowed.0].to_owned(),
                source[borrowed.1].to_owned(),
                source[borrowed.2].to_owned(),
                source[borrowed.3].to_owned(),
            )
        }
    }
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum OwnedItem {
        Section {
            key: String,
            padding: OwnedPadding2,
            line_ending: LineEnding,
        },
        Property {
            key: String,
            value: String,
            padding: OwnedPadding4,
            line_ending: LineEnding,
        },
        Comment {
            comment: String,
            padding: OwnedPadding2,
            line_ending: LineEnding,
        },
        Blank {
            line: String,
            line_ending: LineEnding,
        },
        Error {
            line: String,
            line_ending: LineEnding,
        },
    }
    
    impl OwnedItem {
        pub fn from_borrowed(borrowed: Item, source: &str) -> OwnedItem {
            match borrowed {
                Item::Section { key, padding, line_ending } => OwnedItem::Section {
                    key: source[key].to_owned(),
                    padding: OwnedPadding2::from_borrowed(padding, source),
                    line_ending,
                },
                Item::Property { key, value, padding, line_ending } => OwnedItem::Property {
                    key: source[key].to_owned(),
                    value: source[value].to_owned(),
                    padding: OwnedPadding4::from_borrowed(padding, source),
                    line_ending,
                },
                Item::Comment { comment, padding, line_ending } => OwnedItem::Comment {
                    comment: source[comment].to_owned(),
                    padding: OwnedPadding2::from_borrowed(padding, source),
                    line_ending,
                },
                Item::Blank { line, line_ending } => OwnedItem::Blank {
                    line: source[line].to_owned(),
                    line_ending,
                },
                Item::Error { line, line_ending } => OwnedItem::Error {
                    line: source[line].to_owned(),
                    line_ending,
                },
            }
        }
    }

    macro_rules! padding {
        ($p1:literal, $p2:literal) => {
            OwnedPadding2(
                const_str::repeat!(" ", $p1).to_owned(),
                const_str::repeat!(" ", $p2).to_owned(),
            )
        };
        ($p1:literal, $p2:literal, $p3:literal, $p4:literal) => {
            OwnedPadding4(
                const_str::repeat!(" ", $p1).to_owned(),
                const_str::repeat!(" ", $p2).to_owned(),
                const_str::repeat!(" ", $p3).to_owned(),
                const_str::repeat!(" ", $p4).to_owned(),
            )
        };
    }

    macro_rules! section {
        ($key:literal, pad=$padding:expr, end=$ending:expr) => {
            OwnedItem::Section {
                key: $key.to_owned(),
                padding: $padding,
                line_ending: $ending,
            }
        };
        ($key:literal, end=$ending:expr) => {
            section!($key, pad=OwnedPadding2::default(), end=$ending)
        };
        ($key:literal, pad=$padding:expr) => {
            section!($key, pad=$padding, end=LineEnding::default())
        };
        ($key:literal) => {
            section!($key, pad=OwnedPadding2::default(), end=LineEnding::default())
        };
    }

    macro_rules! prop {
        ($key:literal => $value:literal, pad=$padding:expr, end=$ending:expr) => {
            OwnedItem::Property {
                key: $key.to_owned(),
                value: $value.to_owned(),
                padding: $padding,
                line_ending: $ending,
            }
        };
        ($key:literal => $value:literal, end=$ending:expr) => {
            prop!($key => $value, pad=OwnedPadding4::default(), end=$ending)
        };
        ($key:literal => $value:literal, pad=$padding:expr) => {
            prop!($key => $value, pad=$padding, end=LineEnding::default())
        };
        ($key:literal => $value:literal) => {
            prop!($key => $value, pad=OwnedPadding4::default(), end=LineEnding::default())
        };
    }

    macro_rules! comment {
        ($comment:literal, pad=$padding:expr, end=$ending:expr) => {
            OwnedItem::Comment {
                comment: $comment.to_owned(),
                padding: $padding,
                line_ending: $ending,
            }
        };
        ($comment:literal, end=$ending:expr) => {
            comment!($comment, pad=OwnedPadding2::default(), end=$ending)
        };
        ($comment:literal, pad=$padding:expr) => {
            comment!($comment, pad=$padding, end=LineEnding::default())
        };
        ($comment:literal) => {
            comment!($comment, pad=OwnedPadding2::default(), end=LineEnding::default())
        };
    }

    macro_rules! blank {
        ($line:literal, end=$ending:expr) => {
            OwnedItem::Blank {
                line: $line.to_owned(),
                line_ending: $ending,
            }
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
            OwnedItem::Error {
                line: $line.to_owned(),
                line_ending: $ending,
            }
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
            .map(|item| OwnedItem::from_borrowed(item, source))
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
            .map(|item| OwnedItem::from_borrowed(item, source))
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
            .map(|item| OwnedItem::from_borrowed(item, source))
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
            .map(|item| OwnedItem::from_borrowed(item, source))
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
