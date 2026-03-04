# Whispery Release Process

This document is the single source of truth for shipping Whispery updates to users on macOS and Windows.

## How Releases Work

- Tagging `v*` on this repo triggers `.github/workflows/release.yml`.
- CI builds platform bundles in the private repo release.
- Updater artifacts are copied to the public repo `ChristianTrummer99/whispery-updates`.
- The app checks `latest.json` in the public repo and offers in-app updates.

## One-Time Setup (Maintainers)

1. Ensure the updater public key is set in `src-tauri/tauri.conf.json`:
   - `plugins.updater.pubkey`
2. Ensure these GitHub Secrets exist in the private repo:
   - `TAURI_SIGNING_PRIVATE_KEY`
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
   - `UPDATES_REPO_TOKEN` (must have write access to `ChristianTrummer99/whispery-updates`)
3. Ensure the public updates repo exists and is initialized (at least one commit).

Optional for signed macOS distribution:

- `APPLE_CERTIFICATE`
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY`
- `APPLE_ID`
- `APPLE_PASSWORD`
- `APPLE_TEAM_ID`

## Standard Release Steps

1. Start from `main` and pull latest:

```bash
git checkout main
git pull
```

2. Bump versions:

```bash
npm run release:prep -- <new-version>
```

Example:

```bash
npm run release:prep -- 0.1.14
```

3. Stage and commit:

```bash
git add -A
git commit -m "short release message, bump to <new-version>"
```

4. Push the commit:

```bash
git push origin HEAD
```

5. Create and push tag:

```bash
git tag v<new-version>
git push origin v<new-version>
```

## Monitor Release Workflow

Check recent runs:

```bash
gh run list --workflow Release --limit 5
```

Watch logs:

```bash
gh run watch <run-id>
```

View failed logs quickly:

```bash
gh run view <run-id> --log-failed
```

## Verify Updater Artifacts

After the workflow succeeds, verify the public release:

1. Confirm tag exists in the updates repo:
   - `https://github.com/ChristianTrummer99/whispery-updates/releases/tag/v<new-version>`
2. Confirm `latest.json` is attached.
3. Confirm platform bundles are attached (`.tar.gz`, `.zip`, `.sig`, etc.).
4. Open `latest.json` and verify URLs point to `ChristianTrummer99/whispery-updates` (not the private repo).

Quick CLI check:

```bash
gh release view v<new-version> --repo ChristianTrummer99/whispery-updates
```

## Smoke Test In-App Update

1. Install an older Whispery build.
2. Launch app and use:
   - auto-check on startup, or
   - manual "Check for Updates" button.
3. Confirm native update dialog appears.
4. Install update and relaunch.
5. Confirm new version is running.

## Common Failure Modes

- **No workflow run appears after tag push**
  - Confirm tag format is exactly `v*` (example: `v0.1.14`).
  - Confirm tag was pushed: `git push origin v0.1.14`.
- **Public updater release fails**
  - Check `UPDATES_REPO_TOKEN` permissions and secret value.
- **App cannot update (404 / Not Found)**
  - Confirm `latest.json` and assets exist in `whispery-updates`.
  - Confirm `latest.json` URLs point to the public repo.
- **macOS/Windows updater signature errors**
  - Confirm Tauri signing private key and password are correct and match the configured public key.

