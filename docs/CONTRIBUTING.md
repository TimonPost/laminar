## Workflow

As a team, we adopted the following the workflow agreements. When we begin work on the amethyst_network crate, we’ll use the same agreements. They are focused on maintaining a high level of quality in the code, and for working with a highly distributed team. We’re including them here as some of the other teams may find them of use.

- All warnings produced by `cargo test` are treated as errors by the CI/CD system
- All `clippy` warnings are treated as errors by the CI/CD system
- We use `kcov` to track our code coverage; we do not have a required minimum, rather we use this as a potential indicator of issues
- We included sample code about using the library
- Setting up a benchmarking framework so we can track regressions
- Unit and integration tests, as well as release testing with docker-compose

## Style Guidelines

As a team, we (eventually) agreed on a coherent style for all our work. See this [document](https://github.com/amethyst/laminar/blob/master/docs/CONTRIBUTING.md#code-style) for more information.
Some of the most helpful ones have been:

- Keep PRs small, preferably under 200 lines of code when possible
- Comments should explain why, not what
- You must provide comments for public API
- No hard-coded values
- No panics nor unwraps in non-test code
- `rustfmt` stable release must be used
- `rustfmt` should be done as its own PR, to avoid generating giant PRs that are impossible to review
- We make use of the [forking workflow](https://nl.atlassian.com/git/tutorials/comparing-workflows/forking-workflow)

## Code style

Some code guidelines to keep in mind when contributing to laminar or amethyst-networking
1. Comments
    - Comment all code you’ve added. Things you should comment: types, methods/functions public fields of structs. 
    - Calculations should be documented. Whether it would be in a PR or code. But it must be clear what is done.
    - Public things should get docstring comments using `///`. Non-public things may use `//` comments
    - Keep comments small
    - Don’t create unnecessary comments. They must add value
    - Comments should explain the “why” not the “what”
2. Hard Coding
    - Don't hard code values anywhere
    - Use the ‘NetworkConfig’ type for common network settings, use consts or parameter input
    - Use of lazy_static is acceptable but first make sure you can’t fix the issue in other ways
3. Code markup
    - Keep files small. Better have small files with small pieces of logic than having one file with 1000 lines of logic with multiple types/structs etc. Note that I speak of logic, tests not included
	- No panics/unwraps in the main codebase, but they are accepted in tests
