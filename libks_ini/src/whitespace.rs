pub mod indexed {
    use std::ops::Range;
    use super::owned;
    
    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct Padding2(pub Range<usize>, pub Range<usize>);

    impl Padding2 {
        pub fn into_owned(self, source: &str) -> owned::Padding2 {
            owned::Padding2(
                source[self.0].to_owned(),
                source[self.1].to_owned(),
            )
        }
    }
    
    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct Padding4(pub Range<usize>, pub Range<usize>, pub Range<usize>, pub Range<usize>);
    
    impl Padding4 {
        pub fn into_owned(self, source: &str) -> owned::Padding4 {
            owned::Padding4(
                source[self.0].to_owned(),
                source[self.1].to_owned(),
                source[self.2].to_owned(),
                source[self.3].to_owned(),
            )
        }
    }
}

pub mod owned {
    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct Padding2(pub String, pub String);
    
    impl Padding2 {
        pub fn from_indexed(source: &str, indexed: super::indexed::Padding2) -> Self {
            indexed.into_owned(source)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct Padding4(pub String, pub String, pub String, pub String);
    
    impl Padding4 {
        pub fn from_indexed(source: &str, indexed: super::indexed::Padding4) -> Self {
            indexed.into_owned(source)
        }
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
