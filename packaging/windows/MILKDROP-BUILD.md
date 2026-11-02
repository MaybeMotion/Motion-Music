# Windows MilkDrop build and library source

MilkDrop is part of the default feature set. Use the source/rebuild guide in
`Motion-Music-v10-MilkDrop-source.zip`, published beside the matching application
source on the [v10 release](https://github.com/MaybeMotion/Motion-Music/releases/tag/motion-v10).
That guide identifies all native source revisions, toolchain versions and
relink instructions. The archive includes full projectm-sys/libprojectM source,
vcpkg recipes/patches and the native source used by this build.

For a normal Windows x64 build, install Rust with the MSVC target, Visual Studio
C++ Build Tools, CMake, libclang and vcpkg with `glew:x64-windows-static`.
Set `VCPKG_INSTALLATION_ROOT`, `LIBCLANG_PATH` and `CMAKE_GENERATOR` to match
your tools. The tested build used Rust 1.99.1 and `Visual Studio 18 2026` as the
generator. From the application root, keep `.cargo/config.toml` for static MSVC
runtime linking and configure the included CMake wrapper before building:

```powershell
$env:MOTION_MUSIC_CMAKE = (Get-Command cmake.exe).Source
$motionBuildTools = Join-Path (Get-Location) 'target\build-tools'
New-Item -ItemType Directory -Force $motionBuildTools | Out-Null
$motionCMakeWrapper = Join-Path $motionBuildTools 'milkdrop-cmake.exe'
rustc --edition=2024 -C target-feature=+crt-static packaging/windows/milkdrop-cmake.rs -o $motionCMakeWrapper
if ($LASTEXITCODE -ne 0) { throw 'Could not build the MilkDrop CMake wrapper.' }
$env:CMAKE = $motionCMakeWrapper
$env:CARGO_BUILD_JOBS = '2'
cargo build --locked --release --bin motion-music
```

The argument-preserving native wrapper applies libprojectM's existing
`ENABLE_SYSTEM_PROJECTM_EVAL=OFF` build option. This uses its pinned bundled
evaluator and avoids combining an external debug evaluator with release C
runtime libraries. It forwards build, install, version and other command modes
unchanged. It changes no dependency source or version. A fresh Cargo target
directory makes sure an earlier native configuration is not reused.

Do not add `--no-default-features` when preparing the full MilkDrop release.
That option deliberately produces a build without the engine. Compiler and
build-tool binaries are not part of the source archive.

For local library changes, follow `README-REBUILD.md` in the third-party archive
to point the downstream application's projectm-sys patch at the supplied local
source, modify it, and recompile/relink the full application. No vendor signing
restriction prevents running the resulting executable.

Distribute `THIRD-PARTY-NOTICES.txt` beside the executable in the Windows ZIP.
Publish both matching source archives beside the binary downloads and retain
the licence texts. See [third-party notices](../../THIRD-PARTY-NOTICES.md).
