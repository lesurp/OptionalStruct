#![no_std]

pub use optional_struct_macro::optional_struct;

pub trait Applicable: Sized {
    type Base;

    fn build(self, mut base: Self::Base) -> Self::Base {
        self.apply_to(&mut base);
        base
    }

    fn apply_to(self, base: &mut Self::Base);

    fn apply_to_opt(self, other: &mut Self);

    fn apply(mut self, other: Self) -> Self {
        other.apply_to_opt(&mut self);
        self
    }

    fn can_convert(&self) -> bool;
}
