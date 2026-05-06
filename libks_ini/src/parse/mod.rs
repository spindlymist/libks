mod line;
use line::{Line, next_line};

mod trim;
use trim::{trimmed_range_start, trimmed_range_end};

mod item;
use item::Item;

mod whitespace;
use whitespace::{Padding2, Padding4};

pub struct Parser<'a> {
    rest: Option<&'a str>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            rest: Some(source),
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let rest = self.rest?;
        let Line {
            line_full,
            line_trimmed,
            ws_before,
            ws_after,
            rest,
            line_ending,
        } = next_line(rest);

        let item = match line_trimmed.chars().next() {
            // Section key
            Some('[') => match line_trimmed.chars().last().unwrap() {
                ']' => Item::Section {
                    key: &line_trimmed[1..line_trimmed.len() - 1],
                    padding: Padding2(ws_before, ws_after),
                    line_ending,
                },
                _ => Item::Error {
                    line: line_full,
                    line_ending,
                }
            },
            // Comment
            Some(';') => Item::Comment {
                comment: &line_trimmed[1..],
                padding: Padding2(ws_before, ws_after),
                line_ending,
            },
            // Property
            Some(_) => match memchr::memchr(b'=', line_trimmed.as_bytes()) {
                Some(eq_index) => {
                    let (key, before_eq) = {
                        let untrimmed = &line_trimmed[..eq_index];
                        let end = trimmed_range_end(untrimmed);
                        (&untrimmed[..end], &untrimmed[end..])
                    };
                    let (value, after_eq) = {
                        let untrimmed = &line_trimmed[eq_index + 1..];
                        let start = trimmed_range_start(untrimmed);
                        (&untrimmed[start..], &untrimmed[..start])
                    };
                    Item::Property {
                        key,
                        value,
                        padding: Padding4(ws_before, before_eq, after_eq, ws_after),
                        line_ending,
                    }
                },
                None => Item::Error {
                    line: line_full,
                    line_ending,
                },
            },
            // Blank
            None => Item::Blank {
                line: line_full,
                line_ending,
            },
        };

        self.rest = rest;
        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::{*, item::*, whitespace::*};

    macro_rules! padding {
        ($p1:literal, $p2:literal) => {
            Padding2(
                const_str::repeat!(" ", $p1),
                const_str::repeat!(" ", $p2),
            )
        };
        ($p1:literal, $p2:literal, $p3:literal, $p4:literal) => {
            Padding4(
                const_str::repeat!(" ", $p1),
                const_str::repeat!(" ", $p2),
                const_str::repeat!(" ", $p3),
                const_str::repeat!(" ", $p4),
            )
        };
    }

    macro_rules! section {
        ($key:literal, pad=$padding:expr, end=$ending:expr) => {
            Item::Section {
                key: $key,
                padding: $padding,
                line_ending: $ending,
            }
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
            Item::Property {
                key: $key,
                value: $value,
                padding: $padding,
                line_ending: $ending,
            }
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
            Item::Comment {
                comment: $comment,
                padding: $padding,
                line_ending: $ending,
            }
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
            Item::Blank {
                line: $line,
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
            Item::Error {
                line: $line,
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
        let actual: Vec<_> = parser.collect();
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
        let actual: Vec<_> = parser.collect();
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
        let actual: Vec<_> = parser.collect();
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
        let actual: Vec<_> = parser.collect();
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
