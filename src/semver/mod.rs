use string_cache::DefaultAtom;

pub mod cargo;
pub mod constraints;
pub mod solver;

pub type PackageName = DefaultAtom;
