use anyhow::Result;
use clap::Args;
use hstr::Atom;
use semver::VersionReq;

#[derive(Debug, Args)]
pub struct SolveVersionCommand {}

impl SolveVersionCommand {
    pub async fn run(self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraints {
    /// Only packages with these names will be considered.
    pub candidate_packages: Vec<Atom>,

    /// These packages must be included in the solution.
    pub compatible_packages: Vec<PackageConstraint>,
}

#[derive(Debug, Clone)]
pub struct Solution {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageConstraint {
    pub name: Atom,
    pub range: VersionReq,
}
