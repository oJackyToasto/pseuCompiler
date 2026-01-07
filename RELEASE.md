# Release Guide

## Creating a GitHub Release

### Quick Steps:

1. **Build release binaries:**
   ```bash
   cargo build --release
   ```

2. **Create and push a tag:**
   ```bash
   git tag -a v0.1.0 -m "Release version 0.1.0"
   git push origin v0.1.0
   ```

3. **Create release on GitHub:**
   - Go to: `https://github.com/YOUR_USERNAME/pseuCompiler/releases`
   - Click "Draft a new release"
   - Select the tag you just created
   - Add release title and description
   - Optionally attach binaries:
     - `target/release/pseudocode.exe` (Windows executable)
     - `target/wasm32-unknown-unknown/release/pseudocode_wasm.wasm` (WASM module)

### Using the Script:

```bash
create-release.bat v0.1.0 "Release notes here"
```

### Using GitHub CLI:

If you have GitHub CLI installed:
```bash
gh release create v0.1.0 --title "v0.1.0" --notes "Release notes" target/release/pseudocode.exe
```

## Release Checklist

- [ ] Update version in `Cargo.toml` if needed
- [ ] Build release binaries (`cargo build --release`)
- [ ] Test the release build
- [ ] Create changelog/notes
- [ ] Create and push tag
- [ ] Create release on GitHub
- [ ] Attach binaries to release
- [ ] Update documentation if needed


