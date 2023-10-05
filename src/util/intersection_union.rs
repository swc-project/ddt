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
    use semver::{Comparator, Op, VersionReq};

    use super::Intersect;

    impl Intersect for Comparator {
        /// - None for incompatible.
        /// - Some if the result is a vector.
        type Error = Option<Vec<Self>>;

        fn intersect(self, other: Self) -> Result<Self, Self::Error> {
            if self == other {
                return Ok(self);
            }

            match (self.op, other.op) {
                (Op::Exact, Op::Exact) => Err(None),
                (Op::Wildcard, _) => Ok(other),
                (_, Op::Wildcard) => Ok(self),
                (Op::Exact, _) => Ok(self),
                (_, Op::Exact) => Ok(other),

                _ => {
                    // TODO: Implement more rules

                    Err(Some(vec![self, other]))
                }
            }
        }
    }

    fn fold_into_comparators(
        to: Vec<Comparator>,
        new: Comparator,
    ) -> Result<Vec<Comparator>, Vec<Comparator>> {
        match to.len() {
            0 => Ok(vec![new]),
            1 => {
                let a = to.into_iter().next().unwrap();
                match a.intersect(new) {
                    Err(None) => Err(vec![]),
                    Ok(a) => Ok(vec![a]),
                    Err(Some(a)) => Err(a),
                }
            }
            _ => to.into_iter().try_fold(vec![new], fold_into_comparators),
        }
    }

    impl Intersect for VersionReq {
        type Error = ();

        fn intersect(self, other: Self) -> Result<Self, Self::Error> {
            let comparators = self
                .comparators
                .into_iter()
                .try_fold(other.comparators, fold_into_comparators);

            let comparators = match comparators {
                Ok(v) => v,
                Err(v) => v,
            };

            Ok(VersionReq { comparators })
        }
    }
}
