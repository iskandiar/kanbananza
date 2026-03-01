# Release Process

## Prerequisites

- **Node.js**: nvm 22 (`nvm use 22` before running the script)
- **Rust**: `source ~/.cargo/env` in your shell (required before any cargo commands)
- **Apple Developer Certificate** (optional): Required only if code signing is enabled in `tauri.conf.json`
- **pnpm**: v10+ (for dependency installation)

## Release Steps

1. **Ensure you're on main branch and all changes are committed:**
   ```bash
   git status
   ```

2. **Run the release script with the new version:**
   ```bash
   ./scripts/release.sh 0.1.0
   ```

   The script will:
   - Validate the semver format (X.Y.Z)
   - Update version in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`
   - Build the release binary
   - Print the DMG path on success

3. **Create a Git tag and commit the version bump:**
   ```bash
   git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
   git commit -m "chore: release v0.1.0"
   git tag v0.1.0
   git push origin main --tags
   ```

4. **Create a GitHub Release:**
   - Go to https://github.com/YOUR_REPO/releases/new
   - Select the tag you just created
   - Add release notes describing changes
   - Attach the DMG file from the path printed by the script

## DMG Location

The built DMG is located at:
```
src-tauri/target/release/bundle/dmg/Kanbananza_X.Y.Z_x64.dmg
```

The script will print the exact path after a successful build.

## Important Notes

- **Never commit the DMG** — it should not be checked into git. Add `*.dmg` to `.gitignore` if not already present.
- The script uses `sed` for version updates. On macOS, it uses the BSD sed variant with `-i ''` for in-place editing.
- If code signing fails, ensure your Apple Developer Certificate is valid and available in Keychain.
- The build process requires a clean checkout — uncommitted changes may cause build failures.

## Troubleshooting

- **nvm not found**: Install nvm from https://github.com/nvm-sh/nvm or use Volta as a fallback
- **cargo not found**: Run `source ~/.cargo/env` in your shell
- **DMG not found**: Check build logs for errors; the Tauri build step may have failed
- **sed: extra characters after substitute command**: This indicates a format issue; verify file structure hasn't changed

## Rollback

If you need to undo a release:
```bash
git tag -d v0.1.0
git push origin --delete v0.1.0
```

Then manually revert the version in the three files or reset the commit.
