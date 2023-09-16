pub trait Intersect: Sized {
    type Error;

    fn intersect(self, other: Self) -> Result<Self, Self::Error>;
}

impl<T> Intersect for Option<T>
where
    T: Intersect,
{
    type Error = T::Error;

    fn intersect(self, other: Self) -> Result<Self, Self::Error> {
        Ok(match (self, other) {
            (Some(a), Some(b)) => Some(a.intersect(b)?),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        })
    }
}

mod semver {
    use semver::{Comparator, Op};

    use super::Intersect;

    impl Intersect for Comparator {
        type Error = ();

        fn intersect(self, other: Self) -> Result<Self, Self::Error> {
            if self == other {
                return Ok(self);
            }

            match (self.op, other.op) {
                (Op::Exact, Op::Exact) => Err(()),
                (Op::Exact, _) => Ok(self),
                (_, Op::Exact) => Ok(other),
            }
        }
    }
}
