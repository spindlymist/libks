use crate::span::Span;
    
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Padding2(pub Span, pub Span);

impl Padding2 {
    pub fn into_owned<S: AsRef<str>>(self, source: S) -> Padding2 {
        Padding2(
            self.0.into_owned(source.as_ref()),
            self.1.into_owned(source.as_ref()),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Padding4(pub Span, pub Span, pub Span, pub Span);

impl Padding4 {
    pub fn into_owned<S: AsRef<str>>(self, source: S) -> Padding4 {
        Padding4(
            self.0.into_owned(source.as_ref()),
            self.1.into_owned(source.as_ref()),
            self.2.into_owned(source.as_ref()),
            self.3.into_owned(source.as_ref()),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineEnding {
    None,
    Cr,
    #[default]
    Lf,
    CrLf,
}

impl std::fmt::Display for LineEnding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            LineEnding::None => "",
            LineEnding::Cr => "\r",
            LineEnding::Lf => "\n",
            LineEnding::CrLf => "\r\n",
        })
    }
}
