# ddt

Dudy dev tools.

# Installation

```sh
cargo install ddt
```

## `ddt git`

### `ddt git resolve-conflict`

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
