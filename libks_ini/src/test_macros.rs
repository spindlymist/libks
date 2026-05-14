macro_rules! assert_eq_iter {
    ($a:expr, $b:expr) => {
        for (i, (a, b)) in $a.into_iter().zip($b.into_iter()).enumerate() {
            assert_eq!(a, b, "at index {i}");
        }
    };
}

macro_rules! before {
    ($path:literal) => {
        include_str!(concat!("../../test_data/before/", $path))
    };
}

macro_rules! after {
    ($path:literal) => {
        include_str!(concat!("../../test_data/after/", $path))
    };
}

macro_rules! padding {
    ($p1:literal, $p2:literal) => {
        crate::whitespace::Padding2(
            const_str::repeat!(" ", $p1).into(),
            const_str::repeat!(" ", $p2).into(),
        )
    };
    ($p1:literal, $p2:literal, $p3:literal, $p4:literal) => {
        crate::whitespace::Padding4(
            const_str::repeat!(" ", $p1).into(),
            const_str::repeat!(" ", $p2).into(),
            const_str::repeat!(" ", $p3).into(),
            const_str::repeat!(" ", $p4).into(),
        )
    };
}

#[macro_export]
macro_rules! section {
    ($key:literal, pad=$padding:expr, end=$ending:expr) => {
        crate::item::Item::Section(crate::item::SectionItem {
            key: $key.into(),
            padding: $padding,
            line_ending: $ending,
        })
    };
    ($key:literal, end=$ending:expr) => {
        section!($key, pad=crate::whitespace::Padding2::default(), end=$ending)
    };
    ($key:literal, pad=$padding:expr) => {
        section!($key, pad=$padding, end=crate::whitespace::LineEnding::default())
    };
    ($key:literal) => {
        section!($key, pad=crate::whitespace::Padding2::default(), end=crate::whitespace::LineEnding::default())
    };
}

macro_rules! prop {
    ($key:literal => $value:literal, pad=$padding:expr, end=$ending:expr) => {
        crate::item::Item::Property(crate::item::PropertyItem {
            key: $key.into(),
            value: $value.into(),
            padding: $padding,
            line_ending: $ending,
        })
    };
    ($key:literal => $value:literal, end=$ending:expr) => {
        prop!($key => $value, pad=crate::whitespace::Padding4::default(), end=$ending)
    };
    ($key:literal => $value:literal, pad=$padding:expr) => {
        prop!($key => $value, pad=$padding, end=crate::whitespace::LineEnding::default())
    };
    ($key:literal => $value:literal) => {
        prop!($key => $value, pad=crate::whitespace::Padding4::default(), end=crate::whitespace::LineEnding::default())
    };
}

macro_rules! comment {
    ($comment:literal, pad=$padding:expr, end=$ending:expr) => {
        crate::item::Item::Comment(crate::item::CommentItem {
            comment: $comment.into(),
            padding: $padding,
            line_ending: $ending,
        })
    };
    ($comment:literal, end=$ending:expr) => {
        comment!($comment, pad=crate::whitespace::Padding2::default(), end=$ending)
    };
    ($comment:literal, pad=$padding:expr) => {
        comment!($comment, pad=$padding, end=crate::whitespace::LineEnding::default())
    };
    ($comment:literal) => {
        comment!($comment, pad=crate::whitespace::Padding2::default(), end=crate::whitespace::LineEnding::default())
    };
}

macro_rules! blank {
    ($line:literal, end=$ending:expr) => {
        crate::item::Item::Blank(crate::item::BlankItem {
            line: $line.into(),
            line_ending: $ending,
        })
    };
    ($line:literal) => {
        blank!($line, end=crate::whitespace::LineEnding::default())
    };
    () => {
        blank!("")
    };
}

macro_rules! error {
    ($line:literal, end=$ending:expr) => {
        crate::item::Item::Error(crate::item::ErrorItem {
            line: $line.into(),
            line_ending: $ending,
        })
    };
    ($line:literal) => {
        error!($line, end=crate::whitespace::LineEnding::default())
    };
}

pub(crate) use assert_eq_iter;
pub(crate) use before;
pub(crate) use after;
pub(crate) use padding;
pub(crate) use section;
pub(crate) use prop;
pub(crate) use comment;
pub(crate) use blank;
pub(crate) use error;
