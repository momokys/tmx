use std::ops::{Range, RangeFrom, RangeTo, RangeFull};

pub enum Bound<'a, T> {
    Excluded(&'a T),
    Included(&'a T),
    Unbounded,
}

pub trait RangeArgument<T> {
    fn start(&self) -> Bound<T>;
    fn end(&self) -> Bound<T>;
}

impl <T> RangeArgument<T> for Range<T> {
    fn start(&self) -> Bound<T> {
        Bound::Included(&self.start)
    }

    fn end(&self) -> Bound<T> {
        Bound::Included(&self.end)
    }
}

impl <T> RangeArgument<T> for RangeFrom<T> {
    fn start(&self) -> Bound<T> {
        Bound::Included(&self.start)
    }

    fn end(&self) -> Bound<T> {
        Bound::Unbounded
    }
}

impl <T> RangeArgument<T> for RangeTo<T> {
    fn start(&self) -> Bound<T> {
        Bound::Unbounded
    }

    fn end(&self) -> Bound<T> {
        Bound::Excluded(&self.end)
    }
}

impl <T> RangeArgument<T> for RangeFull {
    fn start(&self) -> Bound<T> {
        Bound::Unbounded
    }

    fn end(&self) -> Bound<T> {
        Bound::Unbounded
    }
}