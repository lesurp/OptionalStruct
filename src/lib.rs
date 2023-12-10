pub use export::optional_struct;

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
