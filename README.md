# ddt

Dudy dev tools.

# Installation

```sh
cargo install ddt
```

## `ddt git`

### `ddt git resolve-lockfile-conflict`

This command allows you to resolve conflicts in lockfiles automatically.

#### Usage

Credit: https://github.com/Praqma/git-merge-driver#documentation

Add a custom merge driver to your **global** gitconfig file. (Typically `~/.gitconfig`)

```gitconfig
[merge "ddt-lockfile"]
	name = A custom merge driver used to resolve conflicts in lockfiles automatically
	driver = ddt git resolve-lockfile-conflict  %O %A %B %L %P

```

then, add some entries to the `.gitattributes` of your project.
You can specify this multiple times.

If your project uses `pnpm` and `cargo` for managing dependencies, you can add this to `.gitattributes`:

```gitattributes
 pnpm.yaml merge=ddt-lockfile
 Cargo.lock merge=ddt-lockfile
```

## `ddt clean`

### Features

- Clean dead git branches.
- Remove **outdated** cargo artifacts.

---

Usage: `ddt clean path/to/dir`

If you run `ddt clean .` from a cargo project using git,
It will remove

- outdated cargo artifacts

This is not perfect, and this currently only removes large files like `.rlib`. Detection of `outdated` depends on `cargo metadata --all-features`. If an artifact for a specific version exists but it's not in dependency graph anymore, it will be removed.

- dead git branches if you pass `--remove-dead-git-branches`

The dead branch is determined by running `git fetch --all`, and branches are removed if upstream tracking branch is gone.
