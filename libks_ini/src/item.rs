use std::fmt;

use crate::{
    span::Span,
    whitespace::{Padding2, Padding4, LineEnding},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Section(SectionItem),
    Property(PropertyItem),
    Comment(CommentItem),
    Blank(BlankItem),
    Error(ErrorItem),
}

impl Item {
    pub fn into_owned<S: AsRef<str>>(self, source: S) -> Item {
        match self {
            Item::Section(inner) => Item::Section(inner.into_owned(source)),
            Item::Property(inner) => Item::Property(inner.into_owned(source)),
            Item::Comment(inner) => Item::Comment(inner.into_owned(source)),
            Item::Blank(inner) => Item::Blank(inner.into_owned(source)),
            Item::Error(inner) => Item::Error(inner.into_owned(source)),
        }
    }
    
    pub fn with_source<'a, S: AsRef<str> + ?Sized>(&'a self, source: &'a S) -> sourced::SourcedItem<'a> {
        match self {
            Item::Section(inner) => sourced::SourcedItem::Section(inner.with_source(source)),
            Item::Property(inner) => sourced::SourcedItem::Property(inner.with_source(source)),
            Item::Comment(inner) => sourced::SourcedItem::Comment(inner.with_source(source)),
            Item::Blank(inner) => sourced::SourcedItem::Blank(inner.with_source(source)),
            Item::Error(inner) => sourced::SourcedItem::Error(inner.with_source(source)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionItem {
    pub key: Span,
    pub padding: Padding2,
    pub line_ending: LineEnding,
}

impl SectionItem {
    pub fn into_owned<S: AsRef<str>>(self, source: S) -> SectionItem {
        SectionItem {
            key: self.key.into_owned(source.as_ref()),
            padding: self.padding.into_owned(source.as_ref()),
            line_ending: self.line_ending,
        }
    }

    
    pub fn with_source<'a, S: AsRef<str> + ?Sized>(&'a self, source: &'a S) -> sourced::SourcedSectionItem<'a> {
        sourced::SourcedSectionItem {
            item: self,
            source: source.as_ref(),
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
    pub key: Span,
    pub value: Span,
    pub padding: Padding4,
    pub line_ending: LineEnding,
}

impl PropertyItem {
    pub fn into_owned<S: AsRef<str>>(self, source: S) -> PropertyItem {
        PropertyItem {
            key: self.key.into_owned(source.as_ref()),
            value: self.value.into_owned(source.as_ref()),
            padding: self.padding.into_owned(source.as_ref()),
            line_ending: self.line_ending,
        }
    }
    
    pub fn with_source<'a, S: AsRef<str> + ?Sized>(&'a self, source: &'a S) -> sourced::SourcedPropertyItem<'a> {
        sourced::SourcedPropertyItem {
            item: self,
            source: source.as_ref(),
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
    pub comment: Span,
    pub padding: Padding2,
    pub line_ending: LineEnding,
}

impl CommentItem {
    pub fn into_owned<S: AsRef<str>>(self, source: S) -> CommentItem {
        CommentItem {
            comment: self.comment.into_owned(source.as_ref()),
            padding: self.padding.into_owned(source.as_ref()),
            line_ending: self.line_ending,
        }
    }
    
    pub fn with_source<'a, S: AsRef<str> + ?Sized>(&'a self, source: &'a S) -> sourced::SourcedCommentItem<'a> {
        sourced::SourcedCommentItem {
            item: self,
            source: source.as_ref(),
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
    pub line: Span,
    pub line_ending: LineEnding,
}

impl BlankItem {
    pub fn into_owned<S: AsRef<str>>(self, source: S) -> BlankItem {
        BlankItem {
            line: self.line.into_owned(source),
            line_ending: self.line_ending,
        }
    }
    
    pub fn with_source<'a, S: AsRef<str> + ?Sized>(&'a self, source: &'a S) -> sourced::SourcedBlankItem<'a> {
        sourced::SourcedBlankItem {
            item: self,
            source: source.as_ref(),
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
    pub line: Span,
    pub line_ending: LineEnding,
}

impl ErrorItem {
    pub fn into_owned<S: AsRef<str>>(self, source: S) -> ErrorItem {
        ErrorItem {
            line: self.line.into_owned(source),
            line_ending: self.line_ending,
        }
    }
    
    pub fn with_source<'a, S: AsRef<str> + ?Sized>(&'a self, source: &'a S) -> sourced::SourcedErrorItem<'a> {
        sourced::SourcedErrorItem {
            item: self,
            source: source.as_ref(),
        }
    }
}

impl From<ErrorItem> for Item {
    fn from(inner: ErrorItem) -> Self {
        Item::Error(inner)
    }
}

pub mod sourced {
    use super::*;
    
    pub enum SourcedItem<'a> {
        Section(SourcedSectionItem<'a>),
        Property(SourcedPropertyItem<'a>),
        Comment(SourcedCommentItem<'a>),
        Blank(SourcedBlankItem<'a>),
        Error(SourcedErrorItem<'a>),
    }
    
    pub struct SourcedSectionItem<'a> {
        pub item: &'a SectionItem,
        pub source: &'a str,
    }
    
    pub struct SourcedPropertyItem<'a> {
        pub item: &'a PropertyItem,
        pub source: &'a str,
    }
    
    pub struct SourcedCommentItem<'a> {
        pub item: &'a CommentItem,
        pub source: &'a str,
    }
    
    pub struct SourcedBlankItem<'a> {
        pub item: &'a BlankItem,
        pub source: &'a str,
    }
    
    pub struct SourcedErrorItem<'a> {
        pub item: &'a ErrorItem,
        pub source: &'a str,
    }
    
    impl<'a> fmt::Display for SourcedItem<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Section(inner) => inner.fmt(f),
                Self::Property(inner) => inner.fmt(f),
                Self::Comment(inner) => inner.fmt(f),
                Self::Blank(inner) => inner.fmt(f),
                Self::Error(inner) => inner.fmt(f),
            }
        }
    }
    
    impl<'a> fmt::Display for SourcedSectionItem<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let SourcedSectionItem { item, source } = self;
            let key = item.key.to_str(source);
            let before = item.padding.0.to_str(source);
            let after = item.padding.1.to_str(source);
            write!(f, "{before}[{key}]{after}{}", item.line_ending)
        }
    }
    
    impl<'a> fmt::Display for SourcedPropertyItem<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let SourcedPropertyItem { item, source } = self;
            let key = item.key.to_str(source);
            let value = item.value.to_str(source);
            let before = item.padding.0.to_str(source);
            let before_eq = item.padding.1.to_str(source);
            let after_eq = item.padding.2.to_str(source);
            let after = item.padding.3.to_str(source);
            write!(f, "{before}{key}{before_eq}={after_eq}{value}{after}{}", item.line_ending)
        }
    }
    
    impl<'a> fmt::Display for SourcedCommentItem<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let SourcedCommentItem { item, source } = self;
            let comment = item.comment.to_str(source);
            let before = item.padding.0.to_str(source);
            let after = item.padding.1.to_str(source);
            write!(f, "{before};{comment}{after}{}", item.line_ending)
        }
    }

    impl<'a> fmt::Display for SourcedBlankItem<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let SourcedBlankItem { item, source } = self;
            let line = item.line.to_str(source);
            write!(f, "{line}{}", item.line_ending)
        }
    }
    
    impl<'a> fmt::Display for SourcedErrorItem<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let SourcedErrorItem { item, source } = self;
            let line = item.line.to_str(source);
            write!(f, "{line}{}", item.line_ending)
        }
    }
}
