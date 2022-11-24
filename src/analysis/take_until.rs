use core::{fmt, iter::FusedIterator};

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct TakeUntil<I, P> {
    iter: I,
    flag: bool,
    predicate: P,
}

pub trait TakeUntilExt: Iterator {
    fn take_until<P>(self, predicate: P) -> TakeUntil<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool;
}

impl<I: Iterator> TakeUntilExt for I {
    fn take_until<P>(self, predicate: P) -> TakeUntil<Self, P>
    where
        Self: Sized,
        P: FnMut(&I::Item) -> bool,
    {
        TakeUntil::new(self, predicate)
    }
}

impl<I: fmt::Debug, P> fmt::Debug for TakeUntil<I, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TakeUntil")
            .field("iter", &self.iter)
            .field("flag", &self.flag)
            .finish()
    }
}

impl<I, P> TakeUntil<I, P>
where
    I: Iterator,
    P: FnMut(&I::Item) -> bool,
{
    pub fn new(iter: I, predicate: P) -> Self {
        Self {
            iter,
            flag: false,
            predicate,
        }
    }
}

impl<I, P> Iterator for TakeUntil<I, P>
where
    I: Iterator,
    P: FnMut(&I::Item) -> bool,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.flag {
            None
        } else {
            self.iter.next().map(|item| {
                if (self.predicate)(&item) {
                    self.flag = true;
                }
                item
            })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.flag {
            (0, Some(0))
        } else {
            (0, self.iter.size_hint().1)
        }
    }
}

impl<I, P> FusedIterator for TakeUntil<I, P>
where
    I: Iterator,
    P: FnMut(&I::Item) -> bool,
{
}
