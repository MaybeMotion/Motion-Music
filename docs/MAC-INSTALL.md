# Install Motion Music on a Mac

The Mac installers are built separately from the Windows portable download.
They include the default MilkDrop/projectM feature.

## Download and install

Open the [Motion Music v12 release](https://github.com/MaybeMotion/Motion-Music/releases/tag/motion-v12)
and expand **Assets**. The Mac installers are added there automatically after
both native Mac builds and package verification succeed. Download the DMG
matching your Mac:

- **Motion-Music-0.14.1-macos-arm64.dmg**: Apple Silicon (M1 or newer).
- **Motion-Music-0.14.1-macos-x86_64.dmg**: Intel.

Find your chip in **Apple menu > About This Mac**. Open the DMG and drag
**Motion Music.app** onto **Applications**. Launch it from
Applications. Spotify Premium is required for local Spotify playback.

The release includes a `.dmg.sha256` file for each installer. Download the one
matching your DMG into the same folder, then run
`shasum -a 256 -c *.dmg.sha256` there to check the download.

These builds are ad-hoc signed, without Apple notarization. macOS may block
the first launch. Only if you trust the source and build, use **System Settings
> Privacy & Security > Open Anyway** after attempting to open the app.
Do not disable Gatekeeper. Organisation-managed Macs may require administrator
approval. A Developer ID signed and notarized public release is a separate step.

The DMGs include the Mac dependency licence notices. Matching application and
MilkDrop library source archives are available from the same v12 release.

## Maintainer builds

**Actions > Build Mac installers > Run workflow** starts native builds for
both Mac architectures. **Publish Mac installers** verifies their source
against the existing v12 release, checks the signed apps and runtime library
dependencies, adds the Mac notices, and attaches the completed DMGs and
checksums to that release. Existing release files are kept. It refuses to
publish changed application source as v12.

The Windows ZIP is a separate download; Mac users need the DMG.

## Build on your Mac

Install Xcode command line tools (`xcode-select --install`), Homebrew, and Rust
through rustup. Then, from this checkout:

```sh
brew install cmake llvm glew pkg-config
export LIBCLANG_PATH="$(brew --prefix llvm)/lib"
rustup show active-toolchain
cargo test --locked --lib --test packaging_macos
bash packaging/macos/build-installer.sh
```

The checkout's `rust-toolchain.toml` selects Rust 1.98.0. The installer and its
SHA-256 checksum are written to `dist/`. Move an existing installer aside before
repeating the same build. The script builds for the Mac running it. It verifies
the app's ad-hoc signature and the resulting disk image. It requires macOS;
Linux cannot execute Apple's bundling, signing, or disk-image tools.

The workflows check native compilation, tests, signing, runtime library paths,
and disk-image integrity. A real Mac launch and Spotify playback remain
separate checks.
