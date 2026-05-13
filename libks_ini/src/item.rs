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
    pub fn into_owned(self, source: &str) -> Item {
        match self {
            Item::Section(inner) => Item::from(inner.into_owned(source)),
            Item::Property(inner) => Item::from(inner.into_owned(source)),
            Item::Comment(inner) => Item::from(inner.into_owned(source)),
            Item::Blank(inner) => Item::from(inner.into_owned(source)),
            Item::Error(inner) => Item::from(inner.into_owned(source)),
        }
    }
    
    pub fn fmt_with_source<S: AsRef<str>>(&self, f: &mut fmt::Formatter, source: S) -> fmt::Result {
        match self {
            Item::Section(inner) => inner.fmt_with_source(f, source),
            Item::Property(inner) => inner.fmt_with_source(f, source),
            Item::Comment(inner) => inner.fmt_with_source(f, source),
            Item::Blank(inner) => inner.fmt_with_source(f, source),
            Item::Error(inner) => inner.fmt_with_source(f, source),
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
    pub fn into_owned(self, source: &str) -> SectionItem {
        SectionItem {
            key: self.key.into_owned(source),
            padding: self.padding.into_owned(source),
            line_ending: self.line_ending,
        }
    }
    
    pub fn fmt_with_source<S: AsRef<str>>(&self, f: &mut fmt::Formatter, source: S) -> fmt::Result {
        let source = source.as_ref();
        let key = self.key.to_str(source);
        let before = self.padding.0.to_str(source);
        let after = self.padding.1.to_str(source);
        write!(f, "{before}[{key}]{after}{}", self.line_ending)
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
    pub fn into_owned(self, source: &str) -> PropertyItem {
        PropertyItem {
            key: self.key.into_owned(source),
            value: self.value.into_owned(source),
            padding: self.padding.into_owned(source),
            line_ending: self.line_ending,
        }
    }
    
    pub fn fmt_with_source<S: AsRef<str>>(&self, f: &mut fmt::Formatter, source: S) -> fmt::Result {
        let source = source.as_ref();
        let key = self.key.to_str(source);
        let value = self.value.to_str(source);
        let before = self.padding.0.to_str(source);
        let before_eq = self.padding.1.to_str(source);
        let after_eq = self.padding.2.to_str(source);
        let after = self.padding.3.to_str(source);
        write!(f, "{before}{key}{before_eq}={after_eq}{value}{after}{}", self.line_ending)
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
    pub fn into_owned(self, source: &str) -> CommentItem {
        CommentItem {
            comment: self.comment.into_owned(source),
            padding: self.padding.into_owned(source),
            line_ending: self.line_ending,
        }
    }

    pub fn fmt_with_source<S: AsRef<str>>(&self, f: &mut fmt::Formatter, source: S) -> fmt::Result {
        let source = source.as_ref();
        let comment = self.comment.to_str(source);
        let before = self.padding.0.to_str(source);
        let after = self.padding.1.to_str(source);
        write!(f, "{before};{comment}{after}{}", self.line_ending)
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
    pub fn into_owned(self, source: &str) -> BlankItem {
        BlankItem {
            line: self.line.into_owned(source),
            line_ending: self.line_ending,
        }
    }
    
    
    pub fn fmt_with_source<S: AsRef<str>>(&self, f: &mut fmt::Formatter, source: S) -> fmt::Result {
        let line = self.line.to_str(source.as_ref());
        write!(f, "{line}{}", self.line_ending)
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
    pub fn into_owned(self, source: &str) -> ErrorItem {
        ErrorItem {
            line: self.line.into_owned(source),
            line_ending: self.line_ending,
        }
    }
    
    
    pub fn fmt_with_source<S: AsRef<str>>(&self, f: &mut fmt::Formatter, source: S) -> fmt::Result {
        let line = self.line.to_str(source.as_ref());
        write!(f, "{line}{}", self.line_ending)
    }
}

impl From<ErrorItem> for Item {
    fn from(inner: ErrorItem) -> Self {
        Item::Error(inner)
    }
}
