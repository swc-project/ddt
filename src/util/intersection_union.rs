pub trait Intersect: Sized {
    fn intersect(self, other: Self) -> Self;
}

impl<T> Intersect for Option<T>
where
    T: Intersect,
{
    fn intersect(self, other: Self) -> Self {
        match (self, other) {
            (Some(a), Some(b)) => Some(a.intersect(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }
}
