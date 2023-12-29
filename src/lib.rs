pub use optional_struct_export::optional_struct;

pub trait Applyable<T> {
    fn apply_to(self, t: &mut T);
}

pub trait OptionalStructWrapperInternal: Sized + TryInto<<Self as OptionalStructWrapperInternal>::Raw, Error=Self> + Applyable<<Self as OptionalStructWrapperInternal>::Raw> {
    type Raw;
}

pub struct OptionalBuilder<T> {
    t: T,
}

pub struct DefaultOptionalBuilder<R> {
    r: R,
}

impl<R> DefaultOptionalBuilder<R> {
    pub fn new(r: R) -> Self {
        DefaultOptionalBuilder { r }
    }
    pub fn apply<T: OptionalStructWrapperInternal<Raw=R>>(mut self, t: T) -> Self {
        t.apply_to(&mut self.r);
        self
    }

    pub fn build(self) -> R {
        self.r
    }
}

impl<R, T: OptionalStructWrapperInternal<Raw=R>> OptionalBuilder<T> {
    pub fn new(t: T) -> Self {
        OptionalBuilder { t }
    }

    pub fn with_default(r: R) -> DefaultOptionalBuilder<R> {
        DefaultOptionalBuilder { r }
    }
    pub fn apply(self, _t: T) -> Self {
        //t.apply_to(&mut self.t);
        self
    }

    pub fn build(self) -> Result<T::Raw, T> {
        self.t.try_into()
    }
}