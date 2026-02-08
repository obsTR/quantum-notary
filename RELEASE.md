# Pushing bug fixes and releasing executables on GitHub

Follow these steps to update the repo with your fixes and publish a release with `.exe` (and other) binaries.

---

## Part 1: Push the fixed code to GitHub

### 1.1 Make sure `target/` is not tracked

A `.gitignore` is in the repo so `target/` (and keys/ledgers) are not committed. If `target/` was added before, remove it from tracking (keeps the folder locally, only stops Git from tracking it):

```powershell
cd C:\Users\onurk\Documents\Projects\quantum-notary
git rm -r --cached target/
```

If Git says `target/` is not in the index, skip this step.

### 1.2 Stage your changes

```powershell
git add .gitignore
git add Cargo.lock
git add README.md
git add src/bin/qs_server.rs
git add src/crypto.rs
git add src/sign_all.rs
git add src/verify.rs
```

Or stage everything except what `.gitignore` excludes:

```powershell
git add -A
git status
```

Confirm that only the files you want are staged (no `target/`, no `*.key`).

### 1.3 Commit with a clear message

```powershell
git commit -m "fix: build errors and README release paths

- qs_server: add Serialize to UploadPayload for JSON serialization
- crypto: bring pqcrypto_traits PublicKey/SecretKey into scope for as_bytes/from_bytes
- verify: bring PublicKey trait into scope for pk.as_bytes()
- sign_all: silence unused dir_root warning
- README: run from project root, Windows paths, where keys are written
- add .gitignore for target/, keys, ledger files"
```

### 1.4 Push to GitHub

```powershell
git push origin main
```

If your default branch is `master`, use `git push origin master` instead.

---

## Part 2: Create a GitHub Release and attach the .exe

Releases are created on GitHub and attach built binaries; you do **not** commit the `.exe` files into the repo.

### 2.1 Build release binaries (on your machine)

From the project root:

```powershell
cd C:\Users\onurk\Documents\Projects\quantum-notary
cargo build --release
```

You will have:

- `target\release\qs_notary.exe`
- `target\release\qs_server.exe`

(On Linux/macOS you get `qs_notary` and `qs_server` without `.exe`.)

### 2.2 Create a tag (optional but recommended)

Tags mark the exact version you are releasing:

```powershell
git tag -a v0.1.0 -m "Release v0.1.0 - bug fixes and README updates"
git push origin v0.1.0
```

Use your desired version (e.g. `v0.1.1` for a patch release).

### 2.3 Create the release on GitHub

1. Open your repo on GitHub.
2. Click **Releases** (right-hand side).
3. Click **Draft a new release** (or **Create a new release**).
4. **Choose a tag:** pick the tag you pushed (e.g. `v0.1.0`) or create the same tag from the UI.
5. **Release title:** e.g. `v0.1.0` or `v0.1.0 – Bug fixes and README`.
6. **Description:** e.g.:

   ```text
   ## Changes
   - Fix build: Serialize for UploadPayload, pqcrypto traits in scope, unused variable
   - README: run from project root, Windows paths, installation and key location
   - Add .gitignore for target and secrets

   ## Downloads
   Attach the Windows executables below (or use the attached assets).
   ```

7. **Attach binaries:**
   - Scroll to the bottom to **Attach binaries by dropping them here or selecting them**.
   - Drag and drop (or select):
     - `qs_notary.exe`
     - `qs_server.exe`
   - You can zip them first if you prefer, e.g. `qs_notary-windows-x64.zip` containing both `.exe` files.
8. Click **Publish release**.

### 2.4 Optional: zip for a single download

From the project root:

```powershell
Compress-Archive -Path .\target\release\qs_notary.exe, .\target\release\qs_server.exe -DestinationPath .\qs_notary-windows-x64-v0.1.0.zip
```

Upload `qs_notary-windows-x64-v0.1.0.zip` to the release as an asset. Users can download one file and unzip to get both executables.

---

## Quick reference

| Goal                    | Command / action |
|-------------------------|------------------|
| Stage all (respecting .gitignore) | `git add -A`     |
| Commit                  | `git commit -m "message"` |
| Push branch             | `git push origin main` (or `master`) |
| Tag                     | `git tag -a v0.1.0 -m "message"` |
| Push tag                | `git push origin v0.1.0` |
| Build release            | `cargo build --release` |
| Release with .exe       | GitHub → Releases → New release → attach `.exe` (or zip) |

---

## Later: automate builds (optional)

To offer Windows/Linux/macOS binaries without building each yourself, you can add **GitHub Actions** to build on push or on release and upload the artifacts to the release. That can be a follow-up step once the manual flow above works for you.
