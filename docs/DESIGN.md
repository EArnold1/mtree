# Merkle Tree CLI

A command-line tool for **directory integrity verification** using Merkle trees.

Unlike traditional directory comparison tools such as `diff` or `rsync`, this CLI generates verifiable snapshots that can be efficiently compared, verified, and synchronized.

---

## Commands

### `mtree build`

```bash
mtree build <directory> > snapshot.json
```

#### Description

Traverses a directory, computes hashes for every file, constructs a Merkle tree, and outputs a snapshot containing the tree and its root hash.

#### Output

A snapshot file (for example, JSON or a binary format) containing:

- Metadata
- Merkle root
- Internal tree nodes (optional)
- File hashes
- File paths

#### Use Cases

- Creating backups
- Recording a trusted state
- Sharing directory fingerprints

---

### `mtree verify`

```bash
mtree verify <directory> snapshot.json
```

#### Description

Rebuilds the Merkle tree for the supplied directory and compares it against a previously generated snapshot.

#### Reports

- Whether the directory is unchanged
- Modified files
- Added files
- Removed files
- Directory structure changes

#### Use Cases

- Integrity checking
- Detecting accidental or malicious modifications
- Verifying restored backups

---

### `mtree diff`

```bash
mtree diff snapshot1.json snapshot2.json
```

#### Description

Compares two snapshots instead of two live directories.

Rather than re-reading every file, the command compares the stored Merkle trees to identify differences.

#### Example Output

```text
Modified:
  docs/readme.md

Added:
  images/logo.png

Removed:
  old/config.toml
```

#### Advantages

- Fast
- Works offline
- Does not require access to the original directories

---

### `mtree proof`

```bash
mtree proof snapshot.json path/to/file
```

#### Description

Generates a Merkle proof showing that a file belongs to a particular directory snapshot.

The proof contains only the hashes necessary to reconstruct the Merkle root from the file's hash.

#### Use Cases

- Remote verification
- Lightweight integrity checks
- Cryptographic membership proofs

---

### `mtree verify-proof`

```bash
mtree verify-proof proof.json
```

#### Description

Verifies a Merkle proof and reconstructs the Merkle root.

This confirms that a file belongs to a directory snapshot without requiring the entire snapshot.

#### Use Cases

- Distributed systems
- Remote file validation
- Secure data exchange

---

### `mtree sync`

```bash
mtree sync <local> <remote>
```

#### Description

Synchronizes two directories by exchanging Merkle trees instead of comparing every file directly.

The algorithm compares hashes from the root downward, descending only into subtrees whose hashes differ.

#### Benefits

- Efficient change detection
- Avoids rehashing identical subtrees
- Transfers only modified files

#### Synchronization Workflow

1. Exchange root hashes.
2. If the roots match, synchronization is complete.
3. Otherwise, compare child hashes.
4. Continue recursively until differing files are identified.
5. Transfer only the required files.

---

## Project Structure

```text
mtree
├── build
├── verify
├── diff
├── proof
├── verify-proof
└── sync
```

---

## Future Enhancements

- Binary snapshot format for improved performance
- Snapshot compression
- Digital signature support
- Incremental snapshot creation
- Parallel hashing
- Ignore patterns (`.gitignore`-style)
- Colored diff output
- Progress indicators for large directories
- Pluggable hash algorithms (SHA-256, BLAKE3, SHA-512)
- Remote synchronization over SSH or libp2p

---

## Design Philosophy

This CLI is **not** intended to replace tools such as `diff` or `rsync`.

Instead, it leverages Merkle trees to provide:

- Efficient cryptographic verification
- Snapshot comparison
- Membership proofs
- Scalable synchronization

These capabilities become especially valuable when working with **large datasets**, **distributed systems**, and **content verification**.
