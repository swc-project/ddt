# ddt

Dudy dev tools.

# Installation

```sh
cargo install ddt
```

# `ddt profile`

Commands to profile your code.

## `ddt profile instruments`

Commands to profile your code using Instruments.app. (macos only)

## `ddt profile instruments cargo`

Example usage:

`ddt profile instruments cargo -t 'Allocations' --release --test snapshot`

This will build a binary using `cargo`, codesign the binary, and run the binary with the `Allocations` instrument in Instruments.app.

# `ddt git`

## `ddt git resolve-conflict`

This command allows you to resolve conflicts in lockfiles automatically.

#### Usage

Credit: https://github.com/Praqma/git-merge-driver#documentation

Add a custom merge driver to your **global** gitconfig file. (Typically `~/.gitconfig`)

```gitconfig
[merge "ddt-auto"]
	name = A custom merge driver used to resolve conflicts in lockfiles automatically
	driver = ddt git resolve-conflict  %O %A %B %L %P

```

then, add some entries to the `.gitattributes` of your project.
You can specify this multiple times.

If your project uses `pnpm` and `cargo` for managing dependencies, you can add this to `.gitattributes`:

```gitattributes
 pnpm.yaml merge=ddt-auto
 Cargo.lock merge=ddt-auto
```
