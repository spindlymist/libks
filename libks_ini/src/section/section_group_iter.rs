use super::{Section, SectionIter};

pub struct SectionGroupIter<'a> {
    sections: Vec<&'a Section>,
    iter: Option<SectionIter<'a>>,
    i: usize,
}

impl<'a> SectionGroupIter<'a> {
    pub fn new(sections: Vec<&'a Section>) -> Self {
        let current_iter = sections.first()
            .map(|section| section.iter());

        Self {
            sections,
            iter: current_iter,
            i: 0,
        }
    }
}

impl<'a> Iterator for SectionGroupIter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let iter = self.iter.as_mut()?;
            if let Some(kvp) = iter.next() {
                return Some(kvp);
            }

            self.i += 1;
            if self.i < self.sections.len() {
                self.iter = Some(self.sections[self.i].iter());
            }
            else {
                self.iter.take();
            }
        }
    }
}
