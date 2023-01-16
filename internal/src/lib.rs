#![feature(specialization)]
pub trait OptionalRepresentation {
    type Inner;
}

default impl<T> OptionalRepresentation for T {
    type Inner = T;
}

pub struct OptionalStructureField<T: OptionalRepresentation>(
    Option<<T as OptionalRepresentation>::Inner>,
);

pub trait Applyable<T> {
    fn apply_to(self, t: &mut T);
}

impl<T> Applyable<T> for Option<T> {
    fn apply_to(self, t: &mut T) {
        if let Some(s) = self {
            *t = s;
        }
    }
}

impl<T> Applyable<T> for T {
    fn apply_to(self, t: &mut T) {
        *t = self;
    }
}

impl<T: OptionalRepresentation> Applyable<T> for OptionalStructureField<T>
where
    Option<<T as OptionalRepresentation>::Inner>: Applyable<T>,
{
    fn apply_to(self, t: &mut T) {
        self.0.apply_to(t)
    }
}
