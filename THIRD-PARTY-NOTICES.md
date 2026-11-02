# Third-party licences and source

Motion Music includes the projectM visualizer and the projectm-sys Rust
bindings, covered by the GNU Lesser General Public License. The application
source is MIT licensed; each third-party component retains its own licence.

The Windows release includes `THIRD-PARTY-NOTICES.txt`, with the original
copyright notices and licence texts for its native libraries and Windows Rust
dependency graph. Copies of LGPL 2.1, LGPL 3 and GPL 3 are included there.
Original upstream declarations are retained, including differing licence
version declarations in the projectM package metadata and licence files.

Download the matching application and library source from the
[Motion Music v11 release](https://github.com/MaybeMotion/Motion-Music/releases/tag/motion-v11):

- `Motion-Music-v11-source.zip`: the application source, assets and Cargo.lock.
- `Motion-Music-v11-MilkDrop-source.zip`: pinned projectm-sys/libprojectM source,
  native dependency source and patches, MPL Rust source, full notices and
  rebuild instructions.

You may modify these libraries and rebuild/relink Motion Music with them.
Reverse engineering for debugging modifications to LGPL components is
permitted. Running a modified executable does not require a vendor signature
or special Motion Music installation key. These components are supplied
without warranty as described in their licences.

See [the Windows MilkDrop build guide](packaging/windows/MILKDROP-BUILD.md).
The third-party source archive contains original license files and a SHA-256
manifest, preserving individual library notices and authorship.
