mod iter;
pub use iter::ItemsIteratorExt;

mod padding;
pub use padding::{Padding, Padding4};

use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Error(Span),
    Section(Span, Padding),
    Property(Prop, Padding4),
    Comment(Span, Padding),
    Blank(Span),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prop {
    pub key: Span,
    pub value: Span,
}

#[derive(Debug, Clone, Copy, Eq)]
pub struct SourcedItem<'a> {
    source: &'a str,
    item: &'a Item,
}

impl Item {
    pub fn with_source<'a>(&'a self, source: &'a str) -> SourcedItem<'a> {
        SourcedItem {
            source,
            item: self,
        }
    }
}

impl<K, V> From<(K, V)> for Prop
where
    K: Into<Span>,
    V: Into<Span>,
{
    fn from(pair: (K, V)) -> Self {
        Self {
            key: pair.0.into(),
            value: pair.1.into(),
        }
    }
}

impl<'a> PartialEq for SourcedItem<'a> {
    fn eq(&self, other: &Self) -> bool { 
        let src = self.source;
        match (self.item, other.item) {
            (Item::Error(span1), Item::Error(span2)) => {
                span1.of(src) == span2.of(src)
            },
            (
                Item::Section(span1, Padding(before1, after1)),
                Item::Section(span2, Padding(before2, after2))
            ) => {
                span1.of(src) == span2.of(src)
                && before1.of(src) == before2.of(src)
                && after1.of(src) == after2.of(src)
            },
            (
                Item::Property(Prop { key: key1, value: value1 }, padding1),
                Item::Property(Prop { key: key2, value: value2 }, padding2),
            ) => {
                key1.of(src) == key2.of(src)
                && value1.of(src) == value2.of(src)
                && padding1.0.of(src) == padding2.0.of(src)
                && padding1.1.of(src) == padding2.1.of(src)
                && padding1.2.of(src) == padding2.2.of(src)
                && padding1.3.of(src) == padding2.3.of(src)
            },
            (
                Item::Comment(span1, Padding(before1, after1)),
                Item::Comment(span2, Padding(before2, after2))
            ) => {
                span1.of(src) == span2.of(src)
                && before1.of(src) == before2.of(src)
                && after1.of(src) == after2.of(src)
            },
            (Item::Blank(span1), Item::Blank(span2)) => {
                span1.of(src) == span2.of(src)
            },
            _ => false,
        }
    }
}

impl<'a> std::fmt::Display for SourcedItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let src = self.source;
        match self.item {
            Item::Error(span) =>
                f.write_str(span.of(src)),
            Item::Section(span, Padding(before, after)) =>
                write!(f, "{}[{}]{}", before.of(src), span.of(src), after.of(src)),
            Item::Property(
                Prop { key, value },
                Padding4(before, before_eq, after_eq, after),
            ) => {
                write!(f, "{}{}{}={}{}{}",
                    before.of(src), key.of(src), before_eq.of(src),
                    after_eq.of(src), value.of(src), after.of(src),
                )
            },
            Item::Comment(span, Padding(before, after)) =>
                write!(f, "{};{}{}", before.of(src), span.of(src), after.of(src)),
            Item::Blank(span) =>
                f.write_str(span.of(src)),
        }
    }
}
