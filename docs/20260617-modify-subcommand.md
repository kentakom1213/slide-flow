Goal:
Refactor the CLI subcommand structure of slide-flow according to the new command tree below.
This repository is still in prerelease, so breaking changes are acceptable. Do not preserve deprecated aliases unless they are trivial and do not complicate the implementation.

New command tree:

slide-flow
├── init
│ └── Initialize a project
│
├── slide
│ ├── add <name>
│ │ ├── --secret
│ │ ├── --public
│ │ ├── --draft
│ │ └── --type <marp|ipe|...>
│ │
│ ├── show <selector>
│ │ └── Show slide details by list number or slide directory
│ │
│ └── archive <dir>
│ └── Archive the current version of the specified slide
│
├── project
│ ├── list
│ │ └── List managed slides
│ │
│ ├── show
│ │ └── Show project configuration and basic project information
│ │
│ └── refresh
│ ├── Update README.md
│ └── Update <output_dir>/index.html
│
├── build
│ ├── <dir>...
│ ├── --all
│ ├── --changed
│ ├── --concurrent <n>
│ └── --no-optimize-images
│
├── prepare
│ ├── <dir>...
│ ├── --all
│ ├── --changed
│ ├── --no-refresh
│ ├── --no-clean
│ ├── --no-toc
│ ├── --no-bib
│ ├── --no-build
│ ├── --no-optimize-images
│ ├── --concurrent <n>
│ └── --dry-run
│
├── toc
│ ├── <dir>...
│ ├── --all
│ ├── --changed
│ └── --quiet
│
├── bib
│ ├── <dir>...
│ ├── --all
│ └── --changed
│
├── images
│ ├── optimize
│ │ ├── <dir>...
│ │ ├── --all
│ │ ├── --changed
│ │ ├── --dry-run
│ │ └── --force
│ │
│ └── clean
│ └── Remove image optimization cache
│
├── clean
│ ├── outputs
│ │ ├── --dry-run
│ │ └── Remove stale generated outputs not included in the current publish plan
│ │
│ └── all
│ ├── --dry-run
│ ├── Remove stale generated outputs
│ └── Remove image optimization cache
│
└── migrate
├── status
├── plan [dir]
└── apply <dir>
├── --metadata-only
├── --redirects-only
├── --artifacts
├── --remove-legacy-artifacts
└── --concurrent <n>

Design principles:

1. `slide-flow slide ...` is only for operations on the slide resource itself, such as adding, showing, and archiving slides.
2. `slide-flow project ...` is for project-level information and project-level generated files, such as `README.md` and `<output_dir>/index.html`.
3. Commands whose target can be explicit slides, changed slides, or all slides should be top-level processing commands. This applies to `build`, `toc`, `bib`, `images optimize`, and `prepare`.
4. Rename the old slide index operation to `toc` to avoid confusion with `<output_dir>/index.html`.
5. Treat `prepare` as the successor of the old `pre-commit` behavior. It should be a composition of refresh, clean outputs, toc, bib, and build.

Target selection:
Introduce a shared target selection structure used by `build`, `toc`, `bib`, `images optimize`, and `prepare`.

Suggested model:

- Explicit target:
  `slide-flow build src/foo src/bar`

- All targets:
  `slide-flow build --all`

- Changed targets:
  `slide-flow build --changed`

Rules:

1. `<dir>...`, `--all`, and `--changed` are mutually exclusive.
2. For `prepare`, no target argument should default to `--changed`.
3. For `build`, `toc`, `bib`, and `images optimize`, require exactly one target mode:
   - explicit directories
   - `--all`
   - `--changed`
4. `--changed` should infer changed slide directories from Git changes under the source directory, following the existing project layout. If reliable changed-target detection is difficult, implement it in a small isolated function and return a clear error when Git is unavailable.
5. Do not run `git add` inside slide-flow. Staging generated files should remain the responsibility of the external hook or user script.

Implementation tasks:

1. Refactor `src/parser.rs`.
   - Replace the current command layout with the new tree.
   - Remove old `slide index` and `slide bib`.
   - Move old `slide list` to `project list`.
   - Keep top-level `build`, `images`, and `migrate`, but update their argument structure to match the new design.
   - Add top-level `toc`, `bib`, `prepare`, and `clean`.
   - Add tests for the new parser behavior.

2. Refactor `src/main.rs`.
   - Update the `match` arms to handle the new subcommands.
   - Avoid duplicating build logic between `build` and `prepare`.
   - Extract reusable functions where necessary.

3. Rework the old `src/subcommand/pre_commit.rs`.
   - Do not expose a `pre-commit` subcommand.
   - Reuse its functionality for `project refresh` and `clean outputs`.
   - Rename functions if appropriate:
     - `create_files` -> `refresh_project_files`
     - `remove_cache` -> `clean_stale_outputs`
   - The name `remove_cache` is misleading because it removes stale generated outputs, not only cache files.

4. Implement `project refresh`.
   - Generate or update `README.md`.
   - Generate or update `<output_dir>/index.html`.
   - Reuse the current templates and existing logic from `pre_commit.rs`.

5. Implement `clean outputs`.
   - Remove stale generated outputs not included in the current publish plan.
   - Support `--dry-run`.
   - Log or print files that would be removed in dry-run mode.
   - Do not remove image cache here.

6. Implement `clean all`.
   - Run `clean outputs`.
   - Run `images clean`.
   - Respect `--dry-run` for output cleanup.
   - For image cache cleanup, either support dry-run or clearly print that image cache cleanup is skipped under dry-run.

7. Implement `toc`.
   - Replace the old `slide index` behavior.
   - Support explicit directories, `--all`, and `--changed`.
   - Preserve `--quiet`.

8. Implement `bib`.
   - Replace the old `slide bib` behavior.
   - Support explicit directories, `--all`, and `--changed`.

9. Implement `build`.
   - Preserve current build behavior, including archived slides, PDF aliases, OGP image generation, image copying, and redirects.
   - Support explicit directories, `--all`, and `--changed`.
   - Preserve `--concurrent`.
   - Preserve `--no-optimize-images`.

10. Implement `prepare`.
    - Default target mode is `--changed`.
    - Run the following steps in order unless disabled:
      1. `project refresh`
      2. `clean outputs`
      3. `toc <targets>`
      4. `bib <targets>`
      5. `build <targets>`
    - Support:
      - `--no-refresh`
      - `--no-clean`
      - `--no-toc`
      - `--no-bib`
      - `--no-build`
      - `--no-optimize-images`
      - `--concurrent <n>`
      - `--dry-run`
    - In `--dry-run`, print the selected target slides and planned steps without writing files or removing files.

11. Implement `project show`.
    - Print basic project information.
    - Include at least:
      - project root
      - output directory
      - number of managed slides
      - source directory if available from configuration
    - Keep output simple and human-readable.

12. Update documentation.
    - Update README command examples.
    - Replace old examples:
      - `slide-flow slide list` -> `slide-flow project list`
      - `slide-flow slide index --dir src/foo` -> `slide-flow toc src/foo`
      - `slide-flow slide index` -> `slide-flow toc --all`
      - `slide-flow slide bib src/foo` -> `slide-flow bib src/foo`
      - `slide-flow pre-commit` -> `slide-flow prepare`
      - `slide-flow images optimize-all` -> `slide-flow images optimize --all`
    - Document recommended hook usage:
      `slide-flow prepare`
      followed by external `git add README.md docs`.

13. Remove obsolete code paths.
    - Since this is prerelease, remove unsupported old subcommands instead of keeping aliases.
    - Ensure there are no stale references to `pre-commit`, `slide index`, `slide bib`, or `images optimize-all` unless they appear in migration notes.

Acceptance criteria:

1. `cargo fmt` passes.
2. `cargo clippy` passes, or all remaining warnings are clearly unrelated to this refactor.
3. `cargo test` passes.
4. `slide-flow --help` shows the new top-level command tree.
5. `slide-flow slide --help` only shows slide resource operations.
6. `slide-flow project --help` shows project-level operations.
7. `slide-flow toc src/foo`, `slide-flow bib src/foo`, and `slide-flow build src/foo` work for explicit targets.
8. `slide-flow toc --all`, `slide-flow bib --all`, and `slide-flow build --all` work for all slides.
9. `slide-flow prepare --dry-run` prints planned steps and targets without modifying files.
10. `slide-flow prepare` defaults to changed slides.
11. No command internally runs `git add`.

Preferred internal structure:

- Keep CLI parsing types in `src/parser.rs`.
- Add a shared target resolution helper, for example:
  - `TargetArgs`
  - `TargetSpec`
  - `resolve_target_slides`
- Keep command execution logic outside `parser.rs`.
- Avoid large duplicated match bodies in `main.rs`.
- If build logic is currently embedded in `main.rs`, extract it into reusable functions so `build` and `prepare` share the same implementation.
