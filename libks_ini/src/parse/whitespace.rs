#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Padding2<'a>(pub &'a str, pub &'a str);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Padding4<'a>(pub &'a str, pub &'a str, pub &'a str, pub &'a str);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineEnding {
    None,
    Cr,
    #[default]
    Lf,
    CrLf,
}
