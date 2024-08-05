mod line;
use line::{Line, next_line};

mod trim;
use trim::{trimmed_range_start, trimmed_range_end};

use crate::item::{Item, Padding, Padding4};

pub struct Parser<'a> {
    remainder: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            remainder: source,
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(Line {
            start_trimmed,
            eq,
            end_trimmed,
            end,
            start_next,
        }) = next_line(self.remainder) else {
            return None;
        };

        let line = &self.remainder[..end];
        let full_line = &self.remainder[..start_next.unwrap_or(self.remainder.len())];
        let trimmed = &self.remainder[start_trimmed..end_trimmed];
        let line_padding = Padding(
            &line[..start_trimmed],
            &full_line[end_trimmed..],
        );

        let item = match trimmed.chars().next() {
            Some('[') => match trimmed.chars().last().unwrap() {
                ']' if trimmed.len() > 2 => {
                    let key = &trimmed[1..trimmed.len() - 1];
                    Item::Section(key.into(), line_padding)
                },
                _ => Item::Error(full_line.into()),
            },
            Some(';') => {
                let comment = &line[start_trimmed + 1..end_trimmed];
                Item::Comment(comment.into(), line_padding)
            },
            Some(_) => match eq {
                Some(eq) => {
                    let (key, before_eq) = {
                        let untrimmed = &self.remainder[start_trimmed..eq];
                        let end_trimmed = trimmed_range_end(untrimmed);
                        (&untrimmed[..end_trimmed], &untrimmed[end_trimmed..])
                    };
                    let (value, after_eq) = {
                        let untrimmed = &self.remainder[eq + 1..end_trimmed];
                        let start_trimmed = trimmed_range_start(untrimmed);
                        (&untrimmed[start_trimmed..], &untrimmed[..start_trimmed])
                    };
                    let padding = Padding4(line_padding.0, before_eq, after_eq, line_padding.1);
                    Item::Property((key, value).into(), padding)
                },
                _ => Item::Error(full_line.into()),
            },
            None => Item::Blank(full_line.into()),
        };

        let start_next = start_next.unwrap_or(self.remainder.len());
        self.remainder = &self.remainder[start_next..];
        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::{Item, Prop};

    macro_rules! prop {
        ( $key:literal => $value:literal ) => {
            Item::Property(
                Prop {
                    key: $key.into(),
                    value: $value.into(),
                },
                Padding4("", "", "", "\n"),
            )
        };
        ( $key:literal => $value:literal; nl = $nl:literal ) => {
            Item::Property(
                Prop {
                    key: $key.into(),
                    value: $value.into(),
                },
                Padding4("", "", "", $nl),
            )
        };
        ( $key:literal => $value:literal; padding = $p1:literal $p2:literal $p3:literal $p4:literal ) => {
            Item::Property(
                Prop {
                    key: $key.into(),
                    value: $value.into(),
                },
                Padding4(
                    const_str::repeat!(" ", $p1),
                    const_str::repeat!(" ", $p2),
                    const_str::repeat!(" ", $p3),
                    const_str::concat!(const_str::repeat!(" ", $p4), "\n"),
                ),
            )
        };
        ( $key:literal => $value:literal; padding = $p1:literal $p2:literal $p3:literal $p4:literal; nl = $nl:literal ) => {
            Item::Property(
                Prop {
                    key: $key.into(),
                    value: $value.into(),
                },
                Padding4(
                    const_str::repeat!(" ", $p1),
                    const_str::repeat!(" ", $p2),
                    const_str::repeat!(" ", $p3),
                    const_str::concat!(const_str::repeat!(" ", $p4), $nl),
                ),
            )
        };
    }

    #[test]
    fn parser_works() {
        let text = "\
;Hello
[World]
Name=The Machine
Author=Nifflas

[x1000y1000]
ShiftVisible(A)=False
ShiftEffect(A)=False
ShiftSound(A)=None";
        let parser = Parser::new(text);
        let items: Vec<_> = parser.collect();

        let lf_after = Padding("", "\n");
        assert_eq!(&items, &[
            Item::Comment("Hello".into(), lf_after),
            Item::Section("World".into(), lf_after),
            prop!["Name" => "The Machine"],
            prop!["Author" => "Nifflas"],
            Item::Blank("\n".into()),
            Item::Section("x1000y1000".into(), lf_after),
            prop!["ShiftVisible(A)" => "False"],
            prop!["ShiftEffect(A)" => "False"],
            prop!["ShiftSound(A)" => "None"; nl=""],
        ]);
        assert_eq!(items.into_iter().map(|item| item.to_string()).collect::<String>(), text);
    }

    #[test]
    fn parser_recognizes_errors() {
        let text = "\
[World] invalid
Name

[x1000y1000
=False";
        let parser = Parser::new(text);
        let items: Vec<_> = parser.collect();

        assert_eq!(&items, &[
            Item::Error("[World] invalid\n".into()),
            Item::Error("Name\n".into()),
            Item::Blank("\n".into()),
            Item::Error("[x1000y1000\n".into()),
            prop!["" => "False"; nl=""],
        ]);
        assert_eq!(items.into_iter().map(|item| item.to_string()).collect::<String>(), text);
    }

    #[test]
    fn parser_handles_whitespace_correctly() {
        let text = "  ;  Hello  
  [World]
  Name=The Machine
Author  =Nifflas
     
[x1000y1000]  
ShiftVisible(A)=  False
ShiftEffect(A)=False  
  ShiftSound(A)  =  None  ";
        let parser = Parser::new(text);
        let items: Vec<_> = parser.collect();
        assert_eq!(&items, &[
            Item::Comment("  Hello".into(), Padding("  ", "  \n")),
            Item::Section("World".into(), Padding("  ", "\n")),
            prop!["Name" => "The Machine"; padding=2 0 0 0],
            prop!["Author" => "Nifflas"; padding=0 2 0 0],
            Item::Blank("     \n".into()),
            Item::Section("x1000y1000".into(), Padding("", "  \n")),
            prop!["ShiftVisible(A)" => "False"; padding=0 0 2 0],
            prop!["ShiftEffect(A)" => "False"; padding=0 0 0 2],
            prop!["ShiftSound(A)" => "None"; padding=2 2 2 2; nl=""],
        ]);
        assert_eq!(items.into_iter().map(|item| item.to_string()).collect::<String>(), text);
    }

    #[test]
    fn parser_handles_newlines_correctly() {
        let text = "\
;Hello\r\
[World]\n\
Name=The Machine\r\n\
Author=Nifflas\r     \n\
[x1000y1000]\r\n\
ShiftVisible(A)=False\r\
ShiftEffect(A)=False\n\
ShiftSound(A)=None\r\n\
";
        let parser = Parser::new(text);
        let items: Vec<_> = parser.collect();

        assert_eq!(&items, &[
            Item::Comment("Hello".into(), Padding("", "\r")),
            Item::Section("World".into(), Padding("", "\n")),
            prop!["Name" => "The Machine"; nl="\r\n"],
            prop!["Author" => "Nifflas"; nl="\r"],
            Item::Blank("     \n".into()),
            Item::Section("x1000y1000".into(), Padding("", "\r\n")),
            prop!["ShiftVisible(A)" => "False"; nl="\r"],
            prop!["ShiftEffect(A)" => "False"; nl="\n"],
            prop!["ShiftSound(A)" => "None"; nl="\r\n"],
        ]);
        assert_eq!(items.into_iter().map(|item| item.to_string()).collect::<String>(), text);
    }
}
