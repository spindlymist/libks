use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Span {
    Range(Range<usize>),
    String(String),
}

impl Default for Span {
    fn default() -> Self {
        Self::Range(0..0)
    }
}

impl Span {
    pub fn into_owned<S: AsRef<str>>(self, source: S) -> Span {
        match self {
            Span::Range(range) => Span::from(&source.as_ref()[range]),
            Span::String(_) => self,
        }
    }
    
    pub fn to_str<'a, S>(&'a self, source: &'a S) -> &'a str
    where
        S: AsRef<str> + ?Sized
    {
        match self {
            Span::Range(range) => &source.as_ref()[range.clone()],
            Span::String(s) => &s,
        }
    }
}

impl From<&str> for Span {
    fn from(value: &str) -> Self {
        Span::String(value.to_owned())
    }
}

impl From<String> for Span {
    fn from(value: String) -> Self {
        Span::String(value)
    }
}

impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        Span::Range(value)
    }
}

impl From<(usize, usize)> for Span {
    fn from((start, end): (usize, usize)) -> Self {
        Span::Range(start..end)
    }
}
