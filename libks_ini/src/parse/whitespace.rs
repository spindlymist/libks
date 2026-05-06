use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Padding2(pub Range<usize>, pub Range<usize>);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Padding4(pub Range<usize>, pub Range<usize>, pub Range<usize>, pub Range<usize>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineEnding {
    None,
    Cr,
    #[default]
    Lf,
    CrLf,
}
