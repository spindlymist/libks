use std::ops::Range;

use super::whitespace::{Padding2, Padding4, LineEnding};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Section {
        key: Range<usize>,
        padding: Padding2,
        line_ending: LineEnding,
    },
    Property {
        key: Range<usize>,
        value: Range<usize>,
        padding: Padding4,
        line_ending: LineEnding,
    },
    Comment {
        comment: Range<usize>,
        padding: Padding2,
        line_ending: LineEnding,
    },
    Blank {
        line: Range<usize>,
        line_ending: LineEnding,
    },
    Error {
        line: Range<usize>,
        line_ending: LineEnding,
    },
}

// impl std::fmt::Display for Item {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         match &self.item {
//             Item::Error(s) =>
//                 f.write_str(s),
//             Item::Section(s, Padding2(before, after)) =>
//                 write!(f, "{}[{}]{}", before, s, after),
//             Item::Property(
//                 Prop { key, value },
//                 Padding4(before, before_eq, after_eq, after),
//             ) => {
//                 write!(f, "{}{}{}={}{}{}",
//                     before, key, before_eq,
//                     after_eq, value, after,
//                 )
//             },
//             Item::Comment(s, Padding2(before, after)) =>
//                 write!(f, "{};{}{}", before, s, after),
//             Item::Blank(s) =>
//                 f.write_str(span),
//         }
//     }
// }
