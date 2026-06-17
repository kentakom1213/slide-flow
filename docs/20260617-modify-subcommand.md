Goal:
Refactor the CLI subcommand structure for slide-flow v0.5.0 and make publish preparation non-destructive by default.

Important safety rule:
`<output_dir>/images/optimized` is public generated output referenced by built HTML. It is not an internal cache. Output pruning must never delete `<output_dir>/images` by default.

New command tree:

```txt
slide-flow
├── init
├── slide
│   ├── add <name> [--secret | --public] [--draft] [--type <marp|ipe|...>]
│   ├── show <selector>
│   └── archive <dir>
├── project
│   ├── list
│   ├── show
│   └── refresh
├── build <dir>... | --all | --changed [--concurrent <n>] [--no-optimize-images]
├── prepare [<dir>... | --all | --changed] [--no-refresh] [--no-toc] [--no-bib] [--no-build] [--no-optimize-images] [--concurrent <n>] [--dry-run]
├── toc <dir>... | --all | --changed [--quiet]
├── bib <dir>... | --all | --changed
├── images
│   ├── optimize <dir>... | --all | --changed [--dry-run] [--force]
│   └── clean
├── prune
│   └── outputs [--dry-run | --apply]
└── migrate
    ├── status
    ├── plan [dir]
    └── apply <dir> [--metadata-only] [--redirects-only] [--artifacts] [--remove-legacy-artifacts] [--concurrent <n>]
```

Design principles:

1. `slide-flow slide ...` is only for slide resource operations: add, show, and archive.
2. `slide-flow project ...` is for project-level information and project-level generated files.
3. `build`, `toc`, `bib`, `images optimize`, and `prepare` share target selection: explicit directories, `--all`, or `--changed`.
4. `prepare` is non-destructive. It runs project refresh, toc, bib, and build, but never output pruning.
5. Destructive stale output deletion is explicit: `slide-flow prune outputs --apply`.
6. `slide-flow prune outputs` defaults to dry-run when neither `--dry-run` nor `--apply` is specified.
7. `slide-flow images clean` removes only the internal image optimization cache.
8. No command internally runs `git add`.

Regression coverage:

- `<output_dir>/images` is retained by stale output detection.
- `<output_dir>/images/optimized/example.png` still exists after `prune outputs --apply`.
- `prepare --dry-run` prints targets and planned steps without writing or deleting files.
