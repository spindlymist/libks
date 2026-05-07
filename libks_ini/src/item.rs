pub mod indexed {
    use std::ops::Range;
    
    use crate::whitespace::{indexed::{Padding2, Padding4}, LineEnding};
    use super::owned::Item as OwnedItem;
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Item {
        Section {
            key: Range<usize>,
            padding: Padding2,
            line_ending: LineEnding,
        },
        Property {
            key: Range<usize>,
            value: Range<usize>,
            padding: Padding4,
            line_ending: LineEnding,
        },
        Comment {
            comment: Range<usize>,
            padding: Padding2,
            line_ending: LineEnding,
        },
        Blank {
            line: Range<usize>,
            line_ending: LineEnding,
        },
        Error {
            line: Range<usize>,
            line_ending: LineEnding,
        },
    }
    
    impl Item {
        pub fn into_owned(self, source: &str) -> OwnedItem {
            match self {
                Item::Section { key, padding, line_ending } => OwnedItem::Section {
                    key: source[key].to_owned(),
                    padding: padding.into_owned(source),
                    line_ending,
                },
                Item::Property { key, value, padding, line_ending } => OwnedItem::Property {
                    key: source[key].to_owned(),
                    value: source[value].to_owned(),
                    padding: padding.into_owned(source),
                    line_ending,
                },
                Item::Comment { comment, padding, line_ending } => OwnedItem::Comment {
                    comment: source[comment].to_owned(),
                    padding: padding.into_owned(source),
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
}

pub mod owned {
    use crate::whitespace::{owned::{Padding2, Padding4}, LineEnding};
    use super::indexed::Item as IndexedItem;
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Item {
        Section {
            key: String,
            padding: Padding2,
            line_ending: LineEnding,
        },
        Property {
            key: String,
            value: String,
            padding: Padding4,
            line_ending: LineEnding,
        },
        Comment {
            comment: String,
            padding: Padding2,
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
    
    impl Item {
        pub fn from_indexed(source: &str, indexed: IndexedItem) -> Item {
            indexed.into_owned(source)
        }
    }
    
    impl std::fmt::Display for Item {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                Item::Section { key, padding: Padding2(before, after), line_ending } =>
                    write!(f, "{before}[{key}]{after}{line_ending}"),
                Item::Property { key, value, padding: Padding4(before, before_eq, after_eq, after), line_ending } =>
                    write!(f, "{before}{key}{before_eq}={after_eq}{value}{after}{line_ending}"),
                Item::Comment { comment, padding: Padding2(before, after), line_ending } =>
                    write!(f, "{before};{comment}{after}{line_ending}"),
                Item::Blank { line, line_ending } =>
                    write!(f, "{line}{line_ending}"),
                Item::Error { line, line_ending } =>
                    write!(f, "{line}{line_ending}"),
            }
        }
    }
}
