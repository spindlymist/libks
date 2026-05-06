use super::whitespace::{Padding2, Padding4, LineEnding};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item<'a> {
    Section {
        key: &'a str,
        padding: Padding2<'a>,
        line_ending: LineEnding,
    },
    Property {
        key: &'a str,
        value: &'a str,
        padding: Padding4<'a>,
        line_ending: LineEnding,
    },
    Comment {
        comment: &'a str,
        padding: Padding2<'a>,
        line_ending: LineEnding,
    },
    Blank {
        line: &'a str,
        line_ending: LineEnding,
    },
    Error {
        line: &'a str,
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
