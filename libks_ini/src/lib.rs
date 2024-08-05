mod section;
mod ini;
mod item;
mod parse;

pub(crate) type Cows<'a> = std::borrow::Cow<'a, str>;

pub use ini::Ini;
pub use parse::Parser;
