pub use optional_struct_export::optional_struct;

pub trait Applyable<T> {
    fn apply_to(self, t: &mut T);
}

pub trait OptionalStructWrapperInternal: Sized + TryInto<<Self as OptionalStructWrapperInternal>::Target, Error=Self> + Applyable<<Self as OptionalStructWrapperInternal>::Target> {
    type Target;
}

pub struct OptionalBuilder<T> {
    opt_struct: T,
}

pub struct DefaultOptionalBuilder<T> {
    default_struct: T,
}

impl<TargetStruct> DefaultOptionalBuilder<TargetStruct> {
    pub fn apply<T: OptionalStructWrapperInternal<Target=TargetStruct>>(mut self, t: T) -> Self {
        t.apply_to(&mut self.default_struct);
        self
    }

    pub fn build(self) -> TargetStruct {
        self.default_struct
    }
}

impl<TargetStruct, OptStruct: OptionalStructWrapperInternal<Target=TargetStruct>> OptionalBuilder<OptStruct> {
    pub fn new(opt_struct: OptStruct) -> Self {
        OptionalBuilder { opt_struct }
    }

    pub fn with_default(default_struct: TargetStruct) -> DefaultOptionalBuilder<TargetStruct> {
        DefaultOptionalBuilder { default_struct }
    }

    pub fn apply<CollapsableStruct: OptionalStructWrapperInternal<Target=OptStruct>>(mut self, c: CollapsableStruct) -> Self {
        c.apply_to(&mut self.opt_struct);
        self
    }

    pub fn build(self) -> Result<OptStruct::Target, OptStruct> {
        self.opt_struct.try_into()
    }
}