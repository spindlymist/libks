pub mod indexed {
    use std::ops::Range;
    
    use crate::whitespace::{indexed::{Padding2, Padding4}, LineEnding};
    use super::owned;
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Item {
        Section(SectionItem),
        Property(PropertyItem),
        Comment(CommentItem),
        Blank(BlankItem),
        Error(ErrorItem),
    }
    
    impl Item {
        pub fn into_owned(self, source: &str) -> owned::Item {
            match self {
                Item::Section(inner) => inner.into_owned(source).into(),
                Item::Property(inner) => inner.into_owned(source).into(),
                Item::Comment(inner) => inner.into_owned(source).into(),
                Item::Blank(inner) => inner.into_owned(source).into(),
                Item::Error(inner) => inner.into_owned(source).into(),
            }
        }
    }
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SectionItem {
        pub key: Range<usize>,
        pub padding: Padding2,
        pub line_ending: LineEnding,
    }
    
    impl SectionItem {
        pub fn into_owned(self, source: &str) -> owned::SectionItem {
            owned::SectionItem {
                key: source[self.key].to_owned(),
                padding: self.padding.into_owned(source),
                line_ending: self.line_ending,
            }
        }
    }
    
    impl From<SectionItem> for Item {
        fn from(inner: SectionItem) -> Self {
            Item::Section(inner)
        }
    }
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct PropertyItem {
        pub key: Range<usize>,
        pub value: Range<usize>,
        pub padding: Padding4,
        pub line_ending: LineEnding,
    }
    
    impl PropertyItem {
        pub fn into_owned(self, source: &str) -> owned::PropertyItem {
            owned::PropertyItem {
                key: source[self.key].to_owned(),
                value: source[self.value].to_owned(),
                padding: self.padding.into_owned(source),
                line_ending: self.line_ending,
            }
        }
    }
    
    impl From<PropertyItem> for Item {
        fn from(inner: PropertyItem) -> Self {
            Item::Property(inner)
        }
    }
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CommentItem {
        pub comment: Range<usize>,
        pub padding: Padding2,
        pub line_ending: LineEnding,
    }
    
    impl CommentItem {
        pub fn into_owned(self, source: &str) -> owned::CommentItem {
            owned::CommentItem {
                comment: source[self.comment].to_owned(),
                padding: self.padding.into_owned(source),
                line_ending: self.line_ending,
            }
        }
    }
    
    impl From<CommentItem> for Item {
        fn from(inner: CommentItem) -> Self {
            Item::Comment(inner)
        }
    }
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BlankItem {
        pub line: Range<usize>,
        pub line_ending: LineEnding,
    }
    
    impl BlankItem {
        pub fn into_owned(self, source: &str) -> owned::BlankItem {
            owned::BlankItem {
                line: source[self.line].to_owned(),
                line_ending: self.line_ending,
            }
        }
    }
    
    impl From<BlankItem> for Item {
        fn from(inner: BlankItem) -> Self {
            Item::Blank(inner)
        }
    }
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ErrorItem {
        pub line: Range<usize>,
        pub line_ending: LineEnding,
    }
    
    impl ErrorItem {
        pub fn into_owned(self, source: &str) -> owned::ErrorItem {
            owned::ErrorItem {
                line: source[self.line].to_owned(),
                line_ending: self.line_ending,
            }
        }
    }
    
    impl From<ErrorItem> for Item {
        fn from(inner: ErrorItem) -> Self {
            Item::Error(inner)
        }
    }
    
}

pub mod owned {
    use std::fmt;
    
    use crate::whitespace::{owned::{Padding2, Padding4}, LineEnding};
    use super::indexed::Item as IndexedItem;
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Item {
        Section(SectionItem),
        Property(PropertyItem),
        Comment(CommentItem),
        Blank(BlankItem),
        Error(ErrorItem),
    }
    
    impl Item {
        pub fn from_indexed(source: &str, indexed: IndexedItem) -> Item {
            indexed.into_owned(source)
        }
    }
    
    impl fmt::Display for Item {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Item::Section(inner) => inner.fmt(f),
                Item::Property(inner) => inner.fmt(f),
                Item::Comment(inner) => inner.fmt(f),
                Item::Blank(inner) => inner.fmt(f),
                Item::Error(inner) => inner.fmt(f),
            }
        }
    }
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SectionItem {
        pub key: String,
        pub padding: Padding2,
        pub line_ending: LineEnding,
    }
    
    impl From<SectionItem> for Item {
        fn from(item: SectionItem) -> Self {
            Item::Section(item)
        }
    }
    
    impl fmt::Display for SectionItem {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let SectionItem {
                key,
                padding: Padding2(before, after),
                line_ending
            } = self;
            write!(f, "{before}[{key}]{after}{line_ending}")
        }
    }
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct PropertyItem {
        pub key: String,
        pub value: String,
        pub padding: Padding4,
        pub line_ending: LineEnding,
    }
    
    impl From<PropertyItem> for Item {
        fn from(item: PropertyItem) -> Self {
            Item::Property(item)
        }
    }
    
    impl fmt::Display for PropertyItem {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let PropertyItem {
                key,
                value,
                padding: Padding4(before, before_eq, after_eq, after),
                line_ending
            } = self;
            write!(f, "{before}{key}{before_eq}={after_eq}{value}{after}{line_ending}")
        }
    }
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CommentItem {
        pub comment: String,
        pub padding: Padding2,
        pub line_ending: LineEnding,
    }
    
    impl From<CommentItem> for Item {
        fn from(item: CommentItem) -> Self {
            Item::Comment(item)
        }
    }
    
    impl fmt::Display for CommentItem {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let CommentItem {
                comment,
                padding: Padding2(before, after),
                line_ending
            } = self;
            write!(f, "{before};{comment}{after}{line_ending}")
        }
    }
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BlankItem {
        pub line: String,
        pub line_ending: LineEnding,
    }
    
    impl From<BlankItem> for Item {
        fn from(item: BlankItem) -> Self {
            Item::Blank(item)
        }
    }
    
    impl fmt::Display for BlankItem {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let BlankItem {
                line,
                line_ending,
            } = self;
            write!(f, "{line}{line_ending}")
        }
    }
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ErrorItem {
        pub line: String,
        pub line_ending: LineEnding,
    }
    
    impl From<ErrorItem> for Item {
        fn from(item: ErrorItem) -> Self {
            Item::Error(item)
        }
    }
    
    impl fmt::Display for ErrorItem {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let ErrorItem {
                line,
                line_ending,
            } = self;
            write!(f, "{line}{line_ending}")
        }
    }
}
