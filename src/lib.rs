#![feature(specialization)]

pub use optional_struct_export::optional_struct;

pub trait Applyable<T> {
    fn apply_to(self, t: &mut T);

    fn can_be_applied(&self) -> bool {
        true
    }
}

impl<T> Applyable<T> for T {
    fn apply_to(self, t: &mut T) {
        *t = self;
    }

    fn can_be_applied(&self) -> bool {
        true
    }
}

impl<T> Applyable<T> for Option<T> {
    fn apply_to(self, t: &mut T) {
        if let Some(s) = self {
            *t = s;
        }
    }

    fn can_be_applied(&self) -> bool {
        self.is_some()
    }
}

pub trait OptionalStructWrapperInternal: Sized + TryInto<<Self as OptionalStructWrapperInternal>::Raw, Error=Self> + Applyable<<Self as OptionalStructWrapperInternal>::Raw> {
    type Raw;
}

pub struct OptionalBuilder<T> {
    t: T,
}

impl<T: OptionalStructWrapperInternal<Raw = T>> OptionalBuilder<T> {
    fn new(t: T) -> Self {
        OptionalBuilder { t }
    }
    fn apply(mut self, t: T) -> Self {
        t.apply_to(&mut self.t);
        self
    }

    fn build(self) -> Result<T::Raw, T> {
        self.t.try_into()
    }
}