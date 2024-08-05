use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Padding(pub Span, pub Span);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Padding4(pub Span, pub Span, pub Span, pub Span);

impl<T, U> From<(T, U)> for Padding
where
    T: Into<Span>,
    U: Into<Span>,
{
    fn from(value: (T, U)) -> Self {
        Self(value.0.into(), value.1.into())
    }
}

impl<T, U, V, W> From<(T, U, V, W)> for Padding4
where
    T: Into<Span>,
    U: Into<Span>,
    V: Into<Span>,
    W: Into<Span>,
{
    fn from(value: (T, U, V, W)) -> Self {
        Self(
            value.0.into(),
            value.1.into(),
            value.2.into(),
            value.3.into(),
        )
    }
}
