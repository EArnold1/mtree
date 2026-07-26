# Verification Next Steps

This is the simplest path to implement `mtree verify` correctly first, then optimize it.

## Goal

Report:

- Unchanged
- Modified files
- Added files
- Removed files
- Directory structure changes

## Phase 1: Correctness First

1. Add snapshot read/write

- Save snapshot as JSON.
- Load snapshot JSON for `verify`.
- Include a `format_version` field.

2. Implement basic verify flow

- Build a fresh snapshot from the live directory.
- Compare root hashes first.
- If roots match: report `unchanged` and exit.

3. File-level diff (works with current data)

- Build maps: `path -> file hash` for old and new snapshot.
- Modified: same path, different hash.
- Added: in new only.
- Removed: in old only.

4. Structure change detection (basic)

- Compare directory path sets.
- Added directories: in new only.
- Removed directories: in old only.

5. Add tests for verify

- Unchanged directory.
- Added file.
- Removed file.
- Modified file.
- Renamed file (removed + added).

## Phase 2: Store Internal Nodes (Optimization + Better Reporting)

1. Persist internal directory nodes in snapshot

- For each directory node store:
  - path
  - node hash
  - child references

2. Update build traversal

- While hashing, also emit internal node records.
- Keep deterministic child order.

3. Hash-pruned verify traversal

- Compare directory node hashes top-down.
- If directory hashes match, skip entire subtree.
- Recurse only into mismatched subtrees.

4. Improve structure reporting

- Detect file <-> directory type changes explicitly.
- Keep output grouped and stable.

## Suggested File Changes

- `src/main.rs`: wire CLI commands for `build` and `verify`.
- `src/lib.rs`: export verify and snapshot I/O modules.
- `src/build.rs`: emit internal node records during traversal.
- New module (example): `src/verify.rs` for comparison logic.
- New module (example): `src/snapshot_io.rs` for JSON load/save.

## Done Criteria

- `mtree build <dir> > snapshot.json` produces a full snapshot.
- `mtree verify <dir> snapshot.json` prints unchanged/modified/added/removed.
- Verify tests pass for file and directory change scenarios.
