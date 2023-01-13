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

impl<T> Applyable<Option<T>> for Option<T> {
    fn apply_to(self, t: &mut Option<T>) {
        *t = self;
    }
}
