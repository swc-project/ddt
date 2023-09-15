use std::sync::Arc;

use anyhow::Result;
use auto_impl::auto_impl;
use gcollections::ops::{Alloc, Bounded, Empty};
use interval::{ops::Range, IntervalSet};
use pcp::{
    concept::Var,
    kernel::Snapshot,
    propagators::{Distinct, XNeqY},
    search::{one_solution_engine, FDSpace, Status, VStore},
    term::Addition,
    variable::Iterable,
};
use semver::{Version, VersionReq};
use string_cache::DefaultAtom;

#[async_trait::async_trait]
#[auto_impl(Arc, Box, &)]
pub trait PackageManager {
    async fn resolve(&self, package_name: &str, constraints: &VersionReq) -> Vec<Version>;
}

pub type PackageName = DefaultAtom;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Constraints {
    /// Only packages with these names will be considered.
    pub candidate_packages: Vec<PackageName>,

    /// These packages must be included in the solution.
    pub compatible_packages: Vec<PackageConstraint>,
}

#[derive(Debug, Clone)]
pub struct Solution {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageConstraint {
    pub name: PackageName,
    pub constraints: VersionReq,
}

pub async fn solve(constraints: Arc<Constraints>) -> Result<Solution> {
    let mut space = FDSpace::empty();

    let mut answer_packages = vec![];

    for wanted in constraints.candidate_packages.iter() {
        answer_packages.push(Box::new(space.vstore.alloc(IntervalSet::new(1, 2))) as Var<VStore>);
    }

    // Search step.
    let mut search = one_solution_engine();
    search.start(&space);
    let (frozen_space, status) = search.enter(space);
    let space = frozen_space.unfreeze();

    // Print result.
    match status {
        Status::Satisfiable => {
            print!("The first solution is:\n[");
            for dom in space.vstore.iter() {
                // At this stage, dom.lower() == dom.upper().
                print!("{}, ", dom.lower());
            }
            println!("]");
        }
        Status::Unsatisfiable => println!("This is unsatisfiable."),
        Status::EndOfSearch => println!("Search terminated or was interrupted."),
        Status::Unknown(_) => unreachable!(
            "After the search step, the problem instance should be either satisfiable or \
             unsatisfiable."
        ),
    }

    Ok(Solution {})
}

pub fn nqueens(n: usize) {
    let mut space = FDSpace::empty();

    let mut queens = vec![];
    // 2 queens can't share the same line.
    for _ in 0..n {
        queens.push(Box::new(space.vstore.alloc(IntervalSet::new(1, n as i32))) as Var<VStore>);
    }
    for i in 0..n - 1 {
        for j in i + 1..n {
            // 2 queens can't share the same diagonal.
            let q1 = (i + 1) as i32;
            let q2 = (j + 1) as i32;
            // Xi + i != Xj + j reformulated as: Xi != Xj + j - i
            space.cstore.alloc(Box::new(XNeqY::new(
                queens[i].bclone(),
                Box::new(Addition::new(queens[j].bclone(), q2 - q1)) as Var<VStore>,
            )));
            // Xi - i != Xj - j reformulated as: Xi != Xj - j + i
            space.cstore.alloc(Box::new(XNeqY::new(
                queens[i].bclone(),
                Box::new(Addition::new(queens[j].bclone(), -q2 + q1)) as Var<VStore>,
            )));
        }
    }
    // 2 queens can't share the same column.
    // join_distinct(&mut space.vstore, &mut space.cstore, queens);
    space.cstore.alloc(Box::new(Distinct::new(queens)));

    // Search step.
    let mut search = one_solution_engine();
    search.start(&space);
    let (frozen_space, status) = search.enter(space);
    let space = frozen_space.unfreeze();

    // Print result.
    match status {
        Status::Satisfiable => {
            print!(
                "{}-queens problem is satisfiable. The first solution is:\n[",
                n
            );
            for dom in space.vstore.iter() {
                // At this stage, dom.lower() == dom.upper().
                print!("{}, ", dom.lower());
            }
            println!("]");
        }
        Status::Unsatisfiable => println!("{}-queens problem is unsatisfiable.", n),
        Status::EndOfSearch => println!("Search terminated or was interrupted."),
        Status::Unknown(_) => unreachable!(
            "After the search step, the problem instance should be either satisfiable or \
             unsatisfiable."
        ),
    }
}
