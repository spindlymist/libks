mod line;
use line::{Line, next_line};

mod trim;
use trim::{trimmed_range_start, trimmed_range_end};

use crate::{
    item::{Item, Padding},
    span::Span,
};

pub struct Parser<'a> {
    source: &'a str,
    start_line: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            start_line: 0,
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        let source = self.source;
        let start_line = self.start_line;

        let Line {
            start_trimmed,
            eq,
            end_trimmed,
            end: _,
            start_next,
        } = next_line(&source[start_line..])?.offset(start_line);

        let line = Span::from(start_line..start_next);
        let trimmed = &source[start_trimmed..end_trimmed];
        let line_padding = Padding::from((
            start_line..start_trimmed,
            end_trimmed..start_next,
        ));

        let item = match trimmed.chars().next() {
            // Section key
            Some('[') => match trimmed.chars().last().unwrap() {
                ']' => {
                    let key = start_trimmed + 1 .. end_trimmed - 1;
                    Item::Section(key.into(), line_padding)
                },
                _ => Item::Error(line),
            },

            // Comment
            Some(';') => {
                let comment = start_trimmed + 1 .. end_trimmed;
                Item::Comment(comment.into(), line_padding)
            },

            // Property
            Some(_) => match eq {
                Some(eq) => {
                    let (key, before_eq) = {
                        let untrimmed = &source[start_trimmed..eq];
                        let end_key = start_trimmed + trimmed_range_end(untrimmed);
                        (start_trimmed..end_key, end_key..eq)
                    };

                    let (value, after_eq) = {
                        let start_untrimmed = eq + 1;
                        let untrimmed = &source[start_untrimmed..end_trimmed];
                        let start_value = start_untrimmed + trimmed_range_start(untrimmed);
                        (start_value..end_trimmed, start_untrimmed..start_value)
                    };

                    Item::Property(
                        (key, value).into(),
                        (line_padding.0, before_eq, after_eq, line_padding.1).into(),
                    )
                },
                _ => Item::Error(line),
            },

            // Blank
            None => Item::Blank(line),
        };

        self.start_line = start_next;
        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::{Item, ItemsIteratorExt, Padding4, Prop};

    macro_rules! prop {
        ( $key:literal => $value:literal ) => {
            Item::Property(
                Prop {
                    key: $key.into(),
                    value: $value.into(),
                },
                Padding4::from(("", "", "", "\n")),
            )
        };
        ( $key:literal => $value:literal; nl = $nl:literal ) => {
            Item::Property(
                Prop {
                    key: $key.into(),
                    value: $value.into(),
                },
                Padding4::from(("", "", "", $nl)),
            )
        };
        ( $key:literal => $value:literal; padding = $p1:literal $p2:literal $p3:literal $p4:literal ) => {
            Item::Property(
                Prop {
                    key: $key.into(),
                    value: $value.into(),
                },
                Padding4::from((
                    const_str::repeat!(" ", $p1),
                    const_str::repeat!(" ", $p2),
                    const_str::repeat!(" ", $p3),
                    const_str::concat!(const_str::repeat!(" ", $p4), "\n"),
                )),
            )
        };
        ( $key:literal => $value:literal; padding = $p1:literal $p2:literal $p3:literal $p4:literal; nl = $nl:literal ) => {
            Item::Property(
                Prop {
                    key: $key.into(),
                    value: $value.into(),
                },
                Padding4::from((
                    const_str::repeat!(" ", $p1),
                    const_str::repeat!(" ", $p2),
                    const_str::repeat!(" ", $p3),
                    const_str::concat!(const_str::repeat!(" ", $p4), $nl)
                )),
            )
        };
    }

    fn items_to_string(items: Vec<Item>, source: &str) -> String {
        items.iter().with_source(source).collect()
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
        let items: Vec<_> = parser.collect();
        let truth = [
            Item::Comment("Hello".into(), ("", "\n").into()),
            Item::Section("World".into(), ("", "\n").into()),
            prop!["Name" => "The Machine"],
            prop!["Author" => "Nifflas"],
            Item::Blank("\n".into()),
            Item::Section("x1000y1000".into(), ("", "\n").into()),
            prop!["ShiftVisible(A)" => "False"],
            prop!["ShiftEffect(A)" => "False"],
            prop!["ShiftSound(A)" => "None"; nl=""],
        ];

        assert_eq!(
            items.iter().with_source(&source).collect::<Vec<_>>(),
            truth.iter().with_source(&source).collect::<Vec<_>>()
        );
        assert_eq!(items_to_string(items, source), source);
    }

    #[test]
    fn parser_recognizes_errors() {
        let source = "\
[World] invalid
Name

[x1000y1000
=False";
        let parser = Parser::new(source);
        let items: Vec<_> = parser.collect();
        let truth = [
            Item::Error("[World] invalid\n".into()),
            Item::Error("Name\n".into()),
            Item::Blank("\n".into()),
            Item::Error("[x1000y1000\n".into()),
            prop!["" => "False"; nl=""],
        ];

        assert_eq!(
            items.iter().with_source(&source).collect::<Vec<_>>(),
            truth.iter().with_source(&source).collect::<Vec<_>>()
        );
        assert_eq!(items_to_string(items, source), source);
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
        let items: Vec<_> = parser.collect();
        let truth = [
            Item::Comment("  Hello".into(), ("  ", "  \n").into()),
            Item::Section("World".into(), ("  ", "\n").into()),
            prop!["Name" => "The Machine"; padding=2 0 0 0],
            prop!["Author" => "Nifflas"; padding=0 2 0 0],
            Item::Blank("     \n".into()),
            Item::Section("x1000y1000".into(), ("", "  \n").into()),
            prop!["ShiftVisible(A)" => "False"; padding=0 0 2 0],
            prop!["ShiftEffect(A)" => "False"; padding=0 0 0 2],
            prop!["ShiftSound(A)" => "None"; padding=2 2 2 2; nl=""],
        ];

        assert_eq!(
            items.iter().with_source(&source).collect::<Vec<_>>(),
            truth.iter().with_source(&source).collect::<Vec<_>>()
        );
        assert_eq!(items_to_string(items, source), source);
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
        let items: Vec<_> = parser.collect();
        let truth = [
            Item::Comment("Hello".into(), ("", "\r").into()),
            Item::Section("World".into(), ("", "\n").into()),
            prop!["Name" => "The Machine"; nl="\r\n"],
            prop!["Author" => "Nifflas"; nl="\r"],
            Item::Blank("     \n".into()),
            Item::Section("x1000y1000".into(), ("", "\r\n").into()),
            prop!["ShiftVisible(A)" => "False"; nl="\r"],
            prop!["ShiftEffect(A)" => "False"; nl="\n"],
            prop!["ShiftSound(A)" => "None"; nl="\r\n"],
        ];

        assert_eq!(
            items.iter().with_source(&source).collect::<Vec<_>>(),
            truth.iter().with_source(&source).collect::<Vec<_>>()
        );
        assert_eq!(items_to_string(items, source), source);
     }
}
