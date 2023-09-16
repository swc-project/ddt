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
                (Op::Exact, _) => Ok(self),
                (_, Op::Exact) => Ok(other),
            }
        }
    }

    fn fold_into_comparators(a: Vec<Comparator>, b: Comparator) -> Vec<Comparator> {
        match a.len() {
            0 => vec![b],
            1 => {
                let a = a.into_iter().next().unwrap();
                match a.intersect(b) {
                    Err(None) => Default::default(),
                    Ok(a) => vec![a],
                    Err(Some(a)) => a,
                }
            }
            _ => a.into_iter().fold(vec![b], fold_into_comparators),
        }
    }

    impl Intersect for VersionReq {
        type Error = ();

        fn intersect(self, other: Self) -> Result<Self, Self::Error> {
            let a = self
                .comparators
                .into_iter()
                .fold(vec![], fold_into_comparators);

            let b_res = other
                .comparators
                .into_iter()
                .try_fold(a_fisrt, |a, b| a.intersect(b));

            let b_res = match b_res {
                Ok(b) => b,
                Err(None) => return Err(()),
                Err(Some(b)) => b,
            };
        }
    }
}
