use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Span {
    Sliced(Range<usize>),
    Owned(String),
}

impl Span {
    pub fn of<'a>(&'a self, s: &'a str) -> &'_ str {
        match self {
            Span::Sliced(range) => &s[range.start .. range.end],
            Span::Owned(value) => value,
        }
    }
}

impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        Self::Sliced(value)
    }
}

impl From<&str> for Span {
    fn from(value: &str) -> Self {
        Self::Owned(value.to_owned())
    }
}

impl From<String> for Span {
    fn from(value: String) -> Self {
        Self::Owned(value)
    }
}
