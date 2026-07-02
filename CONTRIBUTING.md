How to contribute to Mawaqit
==========================

Assalamu alaykum.

Thank you for considering contributing to Mawaqit!

Reporting issues
----------------

- Describe what you expected to happen.
- If possible, include a minimal, complete, and verifiable example to help
  us identify the issue. This also helps check that the issue is not with your
  own code.
- Describe what actually happened. Include the full traceback if there was an
  exception.

Submitting patches
------------------

- Include tests if your patch is supposed to solve a bug, and explain
  clearly under which circumstances the bug happens. Make sure the test fails
  without your patch.
- Make sure all commits are verified.
- Make sure there are no trailing spaces in any of the modified files.
- Run `cargo test` and `cargo fmt` before submitting. Ensure no warnings or errors.

First Time Setup
----------------

Best way to have a local Rust development environment set up is by using [rustup](https://www.rust-lang.org/tools/install).

There are two direct dependencies for this crate:
- [chrono](https://docs.rs/crate/chrono/0.4)
- [spectral](https://docs.rs/spectral/0.6/spectral/) (needed only for running tests.)

These would be installed when you run `cargo build` (or `cargo test`).

Architecture overview
---------------------

- `src/lib.rs` — crate root, re-exports and prelude
- `src/schedule.rs` — prayer time calculation and scheduling
- `src/astronomy/` — solar calculations, coordinates, qiblah direction
- `src/models/` — prayer times, methods, adjustments, madhab, rounding, shafaq

Start coding
------------

- Create a branch to identify the issue you would like to work on (e.g.
  `fix-fajr-times-high-latitude`)
- Using your favorite editor, make your changes, [committing as you go](https://github.com/git-guides/git-commit).
- Make sure there are no trailing spaces in any of the modified files.
- Include tests that cover any code changes you make. Make sure the test fails
  without your patch.
- Push your commits to GitHub and [create a pull request](https://docs.github.com/articles/creating-a-pull-request/).

Running the tests
-----------------

Run the basic test suite with:

    cargo test

If you would only like to run a single test you can do so with:

    cargo test <name of test>

Building the docs
-----------------

Docs for the crate can be locally built using:

    cargo doc --no-deps

Jazakum Allah khayran.
