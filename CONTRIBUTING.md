👋 Hi and welcome to [Artichoke]. Thanks for taking the time to contribute!
💪💎🙌

Artichoke aspires to be a [recent MRI Ruby][mri-target]-compatible
implementation of the Ruby programming language. [There is lots to do].

[mri-target]:
  https://github.com/artichoke/artichoke/blob/trunk/RUBYSPEC.md#mri-target

roe is used to implement Unicode case mapping routines for the [`Symbol`] and
[`String`] classes in Artichoke's implementation of Ruby Core.

If Artichoke does not run Ruby source code in the same way that MRI does, it is
a bug and we would appreciate if you [filed an issue so we can fix it]. [File
bugs specific to roe in this repository].

If you would like to contribute code to roe 👩‍💻👨‍💻, find an issue that looks
interesting and leave a comment that you're beginning to investigate. If there
is no issue, please file one before beginning to work on a PR. [Good first
issues are labeled `E-easy`].

## Setup

roe includes Rust, Ruby, and text sources. Developing on roe requires
configuring several dependencies. [mise] manages the local development toolchain
declared in [`mise.toml`](mise.toml), including Node.js, Ruby, Rust,
`ucd-generate`, and auxiliary Rust tools.

### Rust Toolchain

roe depends on Rust and several compiler plugins for linting and formatting. roe
is guaranteed to build on the latest stable release of the Rust compiler.

#### Installation

Install and activate [mise], then install the toolchains declared in
[`mise.toml`](mise.toml):

```sh
mise install
```

`mise.toml` configures the latest stable Rust toolchain with the `minimal`
profile plus the `clippy` and `rustfmt` components. mise installs that toolchain
via [rustup]. Documentation checks use nightly Rust; install it with
`rustup toolchain install nightly` if you run those workflows locally.

To update your stable Rust compiler to the latest version, run:

```sh
rustup update stable
```

### Rust Crates

roe depends on several Rust libraries, or crates. Once you have the Rust
toolchain installed, you can install the crates specified in
[`Cargo.toml`](Cargo.toml) by running:

```sh
mise run build
```

Common development tasks are declared in [`mise.toml`](mise.toml):

```sh
mise run build
mise run test
mise run fmt
mise run fmt:check
mise run lint
mise run lint:clippy:restriction
mise run doc
mise run doc:open
```

### Ruby and Unicode data

Ruby is only used by dependency-free scripts that update Unicode data and
generate Rust lookup tables. mise installs both Ruby and `ucd-generate`.

```sh
mise run unicode:update
mise run unicode:build
```

The update task downloads the current Unicode license and Unicode Character
Database inputs. Review those changes before rebuilding and committing the
generated Rust code.

### Node.js

Node.js is an optional dependency that is used for formatting text sources with
[prettier].

Node.js is only required for formatting if modifying the following filetypes:

- `md`
- `yaml`
- `yml`

Install Node.js with mise and install the repository-local dependencies:

```sh
mise install
pnpm install
```

## Linting

To lint and check formatting for Rust and text sources run:

```sh
mise run lint
```

The unenforced Clippy restriction lint pass is available separately as
`mise run lint:clippy:restriction`.

## Testing

A PR must have new or existing tests for it to be merged. The [Rust book chapter
on testing] is a good place to start.

To run tests:

```sh
mise run test
```

`cargo test` accepts a filter argument that will limit test execution to tests
that substring match. For example, to run all of the tests for ascii casecmp:

```sh
cargo test ascii
```

Tests are run for every PR. All builds must pass before merging a PR.

## Publishing

Maintainers publish releases through crates.io trusted publishing. See
[`docs/publishing.md`](docs/publishing.md) for the trust configuration, release
procedure, and failure-recovery guidance.

## Updating Dependencies

### Rust Crates

Version specifiers in `Cargo.toml` are NPM caret-style by default. A version
specifier of `4.1.2` means `4.1.2 <= version < 5.0.0`.

To see what crates are outdated, you can use [cargo-outdated].

If you need to pull in an updated version of a crate for a bugfix or a new
feature, update the version number in `Cargo.toml`. See
[artichoke/artichoke#548] for an example.

Regular dependency bumps are handled by [@dependabot].

[artichoke]: https://github.com/artichoke
[there is lots to do]: https://github.com/artichoke/artichoke/issues
[`symbol`]: https://ruby-doc.org/core-3.1.2/Symbol.html
[`string`]: https://ruby-doc.org/core-3.1.2/String.html
[filed an issue so we can fix it]:
  https://github.com/artichoke/artichoke/issues/new
[file bugs specific to roe in this repository]:
  https://github.com/artichoke/roe/issues/new
[good first issues are labeled `e-easy`]:
  https://github.com/artichoke/roe/labels/E-easy
[rustup]: https://rustup.rs/
[mise]: https://mise.jdx.dev/
[prettier]: https://prettier.io/
[node.js]: https://nodejs.org/en/download/package-manager/
[rust book chapter on testing]:
  https://doc.rust-lang.org/book/ch11-00-testing.html
[cargo-outdated]: https://github.com/kbknapp/cargo-outdated
[artichoke/artichoke#548]: https://github.com/artichoke/artichoke/pull/548
[@dependabot]: https://dependabot.com/
