# Contributing

This section is for maintainers, extenders, and contributors.

## In this section

| Chapter | Contents |
|---|---|
| [Design Philosophy](philosophy.md) | Project purpose, threat model, and guiding principles |
| [Architecture](architecture.md) | Workspace layout, crate responsibilities, dependency rules |
| [Local Development](local-development.md) | Setting up the environment and running tests |
| [Reproducible Builds](reproducible-builds.md) | Locked dependencies, build flags, checksum verification |
| [Release Process](release.md) | Release checklist and distribution workflow |

## Before contributing

1. Read [Design Philosophy](philosophy.md) to understand the constraints
   that must not be relaxed under any circumstances.
2. Read [Architecture](architecture.md) to understand where new code belongs.
3. Run `cargo test` and confirm all tests pass before opening a pull request.
4. Run `cargo xtask lint` and confirm there are no warnings.
