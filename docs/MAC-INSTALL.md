# Install Motion Music on a Mac

The Mac installers are built separately from the Windows portable download.
They include the default MilkDrop/projectM feature.

## Download a build

Once these changes are on GitHub, open the repository's **Actions** tab,
choose **Build Mac installers**, and select **Run workflow**. When both jobs
succeed, download the artifact matching your Mac:

- **arm64**: Apple Silicon (M1 or newer).
- **x86_64**: Intel.

Find your chip in **Apple menu > About This Mac**. Unzip the artifact, open
the DMG, and drag **Motion Music.app** onto **Applications**. Launch it from
Applications. Spotify Premium is required for local Spotify playback.

The artifact also contains a checksum. In Terminal, from its extracted folder,
run `shasum -a 256 -c *.dmg.sha256` to check the download.

These builds are ad-hoc signed, without Apple notarization. macOS may block
the first launch. Only if you trust the source and build, use **System Settings
> Privacy & Security > Open Anyway** after attempting to open the app.
Do not disable Gatekeeper. Organisation-managed Macs may require administrator
approval. A Developer ID signed and notarized public release is a separate step.

The workflow uploads build artifacts; it does not publish a GitHub release.
No Mac installer is included in the existing Windows v12 download.

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

Compilation, installation, and playback on macOS still need validation by a
successful Mac workflow and a launch on a real Mac.
