use crate::Cows;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item<'a> {
    Error(Cows<'a>),
    Section(Cows<'a>, Padding<'a>),
    Property(Prop<'a>, Padding4<'a>),
    Comment(Cows<'a>, Padding<'a>),
    Blank(Cows<'a>),
}

impl<'a> std::fmt::Display for Item<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error(s) => write!(f, "{s}"),
            Self::Section(s, Padding(before, after)) => write!(f, "{before}[{s}]{after}"),
            Self::Property(
                    Prop { key, value },
                    Padding4(before, before_eq, after_eq, after),
                ) => write!(f, "{before}{key}{before_eq}={after_eq}{value}{after}"),
            Self::Comment(s, Padding(before, after)) => write!(f, "{before};{s}{after}"),
            Self::Blank(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prop<'a> {
    pub key: Cows<'a>,
    pub value: Cows<'a>,
}

impl<'a> std::fmt::Display for Prop<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.key, self.value)
    }
}

impl<'a> From<(&'a str, &'a str)> for Prop<'a> {
    fn from(pair: (&'a str, &'a str)) -> Self {
        Self {
            key: pair.0.into(),
            value: pair.1.into(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Padding<'a>(pub &'a str, pub &'a str);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Padding4<'a>(pub &'a str, pub &'a str, pub &'a str, pub &'a str);
