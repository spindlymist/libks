use super::{Item, SourcedItem};

pub trait ItemsIteratorExt<'a, I>
where
    I: Iterator<Item = &'a Item>,
{
    fn with_source(self, source: &'a str) -> SourcedItemsIter<'a, I>;
}

pub struct SourcedItemsIter<'a, I>
where
    I: Iterator<Item = &'a Item>,
{
    items: I,
    source: &'a str,
}

impl<'a, I> ItemsIteratorExt<'a, I> for I
where
    I: Iterator<Item = &'a Item>,
{
    fn with_source(self, source: &'a str) -> SourcedItemsIter<'a, I> {
        SourcedItemsIter {
            items: self,
            source,
        }
    }
}

impl<'a, I> Iterator for SourcedItemsIter<'a, I>
where
    I: Iterator<Item = &'a Item>,
{
    type Item = SourcedItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.next()
            .map(|item| item.with_source(self.source))
    }
}

impl<'a> FromIterator<SourcedItem<'a>> for String {
    fn from_iter<T: IntoIterator<Item = SourcedItem<'a>>>(iter: T) -> Self {
        iter.into_iter()
            .map(|item| item.to_string())
            .collect::<String>()
    }
}
