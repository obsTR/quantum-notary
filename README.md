# qs_notary – Post-Quantum SBOM Notary CLI

**qs_notary** is a command-line tool for cryptographically **signing** and **verifying** Software Bills of Materials (SBOMs) using **post-quantum cryptography** (Dilithium5). It helps secure your software supply chain with NIST-standard quantum-resistant signatures, transparency logging, and optional policy-based verification.

---

## Table of Contents

- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Commands Reference](#commands-reference)
  - [generate-keys](#generate-keys)
  - [sign](#sign)
  - [verify](#verify)
  - [sign-all](#sign-all)
- [Transparency Log Server (qs_server)](#transparency-log-server-qs_server)
- [Policy Engine](#policy-engine)
- [Signature Format](#signature-format)
- [License](#license)

---

## Features

- **Post-quantum signing** – Uses **Dilithium5** (NIST PQC standard) for signatures that remain secure against future quantum computers.
- **SBOM support** – Validates and signs **CycloneDX** and **SPDX** JSON SBOMs; refuses to sign invalid or unknown formats.
- **Content binding** – Signs the **SHA3-256** hash of the file so any change invalidates the signature.
- **Key management** – Local key files by default; optional **mock KMS** mode (`--kms`) for testing remote signing.
- **Transparency log** – Local append-only ledger (e.g. `ledger.json`) plus optional **remote log server** (`--server-url`) for centralized audit.
- **Policy-based verification** – Optional **policy file** (`--policy`) to enforce key allowlists and **max signature age**.
- **Batch signing** – **sign-all** recursively signs every file in a directory and produces a signed **manifest** as a root of trust.

---

## Requirements

- **Rust** 1.70+ (install from [rustup.rs](https://rustup.rs))
- **Windows:** Visual Studio Build Tools with "Desktop development with C++" (for MSVC) or MinGW for the GNU toolchain

---

## Installation

Clone the repository and build:

```bash
git clone https://github.com/your-org/quantum-notary.git
cd quantum-notary
cargo build --release
```

Binaries:

- **qs_notary** – CLI (`target/release/qs_notary.exe` on Windows, `target/release/qs_notary` elsewhere)
- **qs_server** – Transparency log server (`target/release/qs_server.exe` / `target/release/qs_server`)

Optional: add `target/release` to your `PATH` or copy the binaries to a directory already in `PATH`.

---

## Quick Start

```bash
# 1. Generate a key pair (writes public.key and private.key in current directory)
qs_notary generate-keys

# 2. Sign an SBOM (e.g. CycloneDX or SPDX JSON)
qs_notary sign sbom.json --private-key private.key

# 3. Verify the signed SBOM
qs_notary verify sbom.json sbom.json.sig --public-key public.key
```

Successful verification prints **Verified Safe** in green; failure prints **Verification Failed** in red.

---

## Commands Reference

### generate-keys

Generate a Dilithium5 key pair and write `public.key` and `private.key` to disk.

| Argument / flag      | Description |
|----------------------|-------------|
| `--output-dir <DIR>` | Directory for key files (default: current directory) |

**Examples:**

```bash
qs_notary generate-keys
qs_notary generate-keys --output-dir ./keys
```

---

### sign

Sign a single SBOM file. Validates that the file is valid JSON and contains CycloneDX (`bomFormat`) or SPDX (`spdxVersion`) markers before signing. Writes a `.sig` file next to the SBOM and appends an entry to the local ledger (and optionally sends it to a server).

| Argument / flag           | Required | Description |
|---------------------------|----------|-------------|
| `SBOM`                    | Yes      | Path to the SBOM file (e.g. `sbom.json`) |
| `-k, --private-key <PATH>`| Yes*     | Path to the private key file (*ignored if `--kms` is set) |
| `--kms`                   | No       | Use mock KMS (in-memory key, 100ms delay) for testing |
| `--ledger <PATH>`         | No       | Ledger file path (default: `ledger.json`) |
| `--server-url <URL>`      | No       | Transparency log server URL (e.g. `http://localhost:8080`); uploads entry in background; signing does not fail if server is unreachable |

**Examples:**

```bash
qs_notary sign sbom.json --private-key private.key
qs_notary sign sbom.json -k private.key --server-url http://localhost:8080
qs_notary sign sbom.json --private-key private.key --ledger my_ledger.jsonl
qs_notary sign sbom.json --kms   # mock KMS (test only; use matching public key for verify)
```

**Output:** Creates `sbom.json.sig` (or `<name>.<ext>.sig` for other extensions) and appends one line to the ledger.

---

### verify

Verify an SBOM file against a signature and public key. Recomputes the file hash and checks the Dilithium5 signature. Optionally applies a policy (key allowlist, max age).

| Argument / flag            | Required | Description |
|----------------------------|----------|-------------|
| `SBOM`                     | Yes      | Path to the original SBOM file |
| `SIGNATURE`                | Yes      | Path to the signature file (e.g. `sbom.json.sig`) |
| `-k, --public-key <PATH>`  | Yes      | Path to the public key file |
| `--policy <PATH>`          | No       | Path to policy JSON; enforces allowlist and/or max_age when set |

**Examples:**

```bash
qs_notary verify sbom.json sbom.json.sig --public-key public.key
qs_notary verify sbom.json sbom.json.sig -k public.key --policy policy.json
```

**Exit / output:** Prints **Verified Safe** (green) on success; **Verification Failed** (red) and exits with an error if the signature is invalid or the policy fails.

---

### sign-all

Recursively sign all files in a directory, then create and sign a **manifest** at the root of that directory. Skips hidden files (names starting with `.`) and existing `.sig` files. Does not validate SBOM format (any file can be signed).

| Argument / flag           | Required | Description |
|---------------------------|----------|-------------|
| `DIR`                     | Yes      | Directory to walk (recursive) |
| `-k, --private-key <PATH>`| Yes*     | Path to the private key (*ignored if `--kms` is set) |
| `--kms`                   | No       | Use mock KMS (test only) |
| `--ledger <PATH>`         | No       | Ledger file (default: `ledger.json`) |
| `--server-url <URL>`      | No       | Transparency log server; each signed file triggers a background upload |

**Examples:**

```bash
qs_notary sign-all ./dist --private-key private.key
qs_notary sign-all ./artifacts -k private.key --server-url http://localhost:8080
```

**Output:**

- For each file: creates `<file>.<ext>.sig` (wrapped format with timestamp).
- Writes **manifest.json** in `DIR` with `entries: [{ "path": "relative/path", "signature_hash": "hex" }, ...]`.
- Signs **manifest.json** and writes **manifest.json.sig** (root of trust for the directory).

---

## Transparency Log Server (qs_server)

**qs_server** is a separate binary that runs an HTTP server for a shared transparency log. The CLI can send ledger entries to it after signing (see `--server-url`).

**Run the server:**

```bash
qs_server
```

- Listens on **0.0.0.0:8080**.
- **POST /upload** – Body: JSON `{ "file_name", "signature_hash", "timestamp" }`. Appends one JSON line to **central_ledger.jsonl** in the server’s current working directory.
- Returns **200** on success, **400** for invalid JSON, **500** on write error.

**Example with CLI:**

```bash
# Terminal 1
qs_server

# Terminal 2
qs_notary sign sbom.json --private-key private.key --server-url http://localhost:8080
```

If the server is unreachable, the sign command logs a warning and still completes (local ledger and `.sig` are still written).

---

## Policy Engine

Use **--policy &lt;FILE&gt;** with **verify** to enforce:

- **allowed_public_keys** – Only the listed public keys (hex-encoded) are accepted.
- **max_age_days** – Signatures older than this many days are rejected (requires a timestamp in the signature; see [Signature Format](#signature-format)).
- **allow_expired** – If `true`, `max_age_days` is not enforced.

**policy.json example:**

```json
{
  "allow_expired": false,
  "max_age_days": 90,
  "allowed_public_keys": [
    "a1b2c3d4e5f6...hex-encoded-public-key-bytes..."
  ]
}
```

- Omit fields or use `null` for “no restriction.”
- Get the hex for your public key once (e.g. `hex::encode(public_key.as_bytes())` or from your tooling) and add it to `allowed_public_keys`.

**Policy failure messages:**

- `Verification failed: public key not in policy allowlist.`
- `Verification failed: signature has no timestamp; cannot apply max_age_days.`
- `Verification failed: signature older than max_age_days.`

---

## Signature Format

- **New signatures** are stored in a **wrapped** format: the `.sig` file is JSON  
  `{ "signature": "<hex>", "timestamp": "<RFC3339>" }`.
- **Legacy** `.sig` files that are raw binary are still supported; verify treats them as having no timestamp (policy `max_age_days` will fail if required).

---

## Project Structure (for developers)

| Path                 | Purpose |
|----------------------|--------|
| `src/main.rs`        | CLI entrypoint, subcommands |
| `src/crypto.rs`      | Dilithium5 keypair, sign/verify, load/save keys and signatures |
| `src/key_provider.rs`| KeyProvider trait, FileSystemProvider, MockKmsProvider |
| `src/sign.rs`        | sign command: SBOM validation, hash, sign, ledger, optional server upload |
| `src/verify.rs`      | verify command: load sig (wrapped or raw), crypto verify, policy checks |
| `src/sign_all.rs`    | sign-all: recursive walk, sign each file, manifest, sign manifest |
| `src/ledger.rs`      | Append-only local ledger (JSON Lines) |
| `src/policy.rs`      | Policy load and fields |
| `src/bin/qs_server.rs` | HTTP server for POST /upload → central_ledger.jsonl |

---

## License

See the repository’s LICENSE file. This project uses post-quantum cryptography (Dilithium5) for SBOM signing and verification in supply chain security workflows.
