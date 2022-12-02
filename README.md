# ddt

Dudy dev tools.

# Features

- Clean dead git branches.
- Remove **outdated** cargo artifacts.

## `ddt clean`

Usage: `ddt clean path/to/dir`

If you run `ddt clean .` from a cargo project using git,
It will remove

- dead git branches

The dead branch is determined by running `git fetch --all`, and branches are removed if upstream tracking branch is gone.

- outdated cargo artifacts

This is not perfect, and this currently only removes large files like `.rlib`. Detection of `outdated` depends on `cargo metadata --all-features`. If an artifact for a specific version exists but it's not in dependency graph anymore, it will be removed.
