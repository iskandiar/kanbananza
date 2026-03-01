Release Kanbananza at version $ARGUMENTS.

Follow these steps exactly:

1. **Validate** that `$ARGUMENTS` matches semver format `X.Y.Z`. If not, stop and tell the user.

2. **Check working tree is clean:**
   Run `git status` — if there are uncommitted changes, stop and warn the user.

3. **Run the release script:**
   ```
   ./scripts/release.sh $ARGUMENTS
   ```
   Wait for it to complete. If it fails, report the error and stop.

4. **Commit the version bump:**
   ```
   git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
   git commit -m "chore: release v$ARGUMENTS"
   ```

5. **Tag and push:**
   ```
   git tag v$ARGUMENTS
   git push origin main --tags
   ```

6. **Report the DMG path** printed by the release script and remind the user to:
   - Create a GitHub Release at https://github.com/iskandiar/kanbananza/releases/new
   - Select tag `v$ARGUMENTS`
   - Attach the DMG (never commit it to git)
