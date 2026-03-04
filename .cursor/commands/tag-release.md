# Tag Release Runbook

Use this command whenever preparing and tagging a new Whispery release.

## Preconditions

- `main` is clean and up to date.
- Required secrets are present in GitHub repo settings:
  - `TAURI_SIGNING_PRIVATE_KEY`
  - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
  - `UPDATES_REPO_TOKEN`
  - macOS signing/notarization: `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`

## Steps

1. Sync branch:

```bash
git checkout main
git pull
```

2. Bump versions:

```bash
npm run release:prep -- <version>
```

3. Stage and commit intended files:

```bash
git add -A
git commit -m "your release message, bump to <version>"
```

4. Push commit:

```bash
git push origin HEAD
```

5. Create and push tag:

```bash
git tag v<version>
git push origin v<version>
```

6. Verify workflow triggered:

```bash
gh run list --workflow Release --limit 5
```

7. Monitor in 5-minute intervals until terminal state:

```bash
sleep 300 && gh run list --workflow Release --limit 5
```

Repeat until the release run for `v<version>` is `completed`.

8. If failed, inspect logs:

```bash
gh run view <run-id> --log-failed
```

9. Verify public updater release assets:

```bash
gh release view v<version> --repo ChristianTrummer99/whispery-updates --json assets,name,tagName
```

## Expected Outcome

- Private release exists for `v<version>`.
- Public updater release exists for `v<version>` in `ChristianTrummer99/whispery-updates`.
- `latest.json` is present and points to public repo assets.
