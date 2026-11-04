#!/usr/bin/env python3
"""Package verified native Mac images with their accompanying notices on Linux.

The DMGs and builder checksum files are preserved byte-for-byte. The native
builder already verified codesign, hdiutil and executable startup on macOS.
Here 7zip extracts the executable for independent architecture/import checks;
no Mach-O code is executed and the signed application is never changed.
"""

import argparse
import hashlib
from pathlib import Path
import posixpath
import re
import shutil
import struct
import subprocess
import sys
import tempfile
import zipfile


ARCHITECTURES = {"arm64": 0x0100000C, "x86_64": 0x01000007}
DYLIB_COMMANDS = {0xC, 0x80000018, 0x8000001F, 0x20, 0x80000023}
NOTICE_NAME = "Motion-Music-v12-macos-THIRD-PARTY-NOTICES.txt"
MANIFEST_NAME = "SHA256-v12-macos.txt"


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def inspect_macho(path, architecture):
    """Read a thin 64-bit executable's bounded Mach-O load commands."""
    with path.open("rb") as stream:
        header = stream.read(32)
        if len(header) != 32:
            raise ValueError("Truncated Mach-O header")
        endian = {b"\xcf\xfa\xed\xfe": "<", b"\xfe\xed\xfa\xcf": ">"}.get(header[:4])
        if endian is None:
            raise ValueError("Expected a thin 64-bit Mach-O executable")
        _, cpu, _, kind, count, size, _, _ = struct.unpack(endian + "IiiIIIII", header)
        if cpu != ARCHITECTURES[architecture]:
            raise ValueError(f"Wrong Mach-O architecture for {architecture}: CPU type {cpu:#x}")
        if kind != 2:
            raise ValueError("Expected MH_EXECUTE Mach-O file type")
        if count > size // 8 or size > path.stat().st_size - 32:
            raise ValueError("Invalid Mach-O load-command bounds")
        commands = stream.read(size)
    position = 0
    libraries = []
    for _ in range(count):
        if position + 8 > size:
            raise ValueError("Truncated Mach-O load command")
        command, length = struct.unpack_from(endian + "II", commands, position)
        if length < 8 or length % 8 or position + length > size:
            raise ValueError("Invalid Mach-O load-command size")
        if command in DYLIB_COMMANDS:
            if length < 24:
                raise ValueError("Truncated Mach-O dylib command")
            offset = struct.unpack_from(endian + "I", commands, position + 8)[0]
            if not 24 <= offset < length:
                raise ValueError("Invalid Mach-O dylib name offset")
            name = commands[position + offset : position + length]
            if b"\0" not in name:
                raise ValueError("Unterminated Mach-O dylib name")
            library = name.split(b"\0", 1)[0].decode("utf-8")
            normalized = posixpath.normpath(library)
            if not normalized.startswith(("/System/Library/", "/usr/lib/")):
                raise ValueError(f"Non-system library requires bundling: {library}")
            libraries.append(library)
        position += length
    if position != size:
        raise ValueError("Mach-O load-command count/size disagree")
    if not libraries:
        raise ValueError("Executable has no identifiable Apple system dylib imports")
    return libraries


def extract_executable(image, destination, sevenzip):
    def extract(source, directory):
        directory.mkdir()
        subprocess.run(
            [
                sevenzip, "x", "-y", f"-o{directory}", str(source),
                "-ir!motion-music", "-ir!*.hfs", "-ir!*.hfsx",
                "-ir!*.img", "-ir!*.apfs", "-xr!Applications",
            ],
            check=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        )

    def executables(directory):
        return [
            path for path in directory.rglob("motion-music")
            if path.is_file() and not path.is_symlink()
            and path.parent.name == "MacOS" and path.parent.parent.name == "Contents"
            and path.parent.parent.parent.name == "Motion Music.app"
        ]

    first = destination / "image"
    extract(image, first)
    candidates = executables(first)
    if not candidates:
        # Some 7zip versions unpack the UDIF layer as a filesystem image.
        partitions = sorted(
            path for path in first.rglob("*")
            if path.is_file() and not path.is_symlink()
            and path.suffix.lower() in {".hfs", ".hfsx", ".img", ".apfs"}
        )
        if not 1 <= len(partitions) <= 8:
            raise ValueError("DMG extraction did not yield an app or a supported filesystem image")
        for index, partition in enumerate(partitions):
            directory = destination / f"partition-{index}"
            extract(partition, directory)
            candidates.extend(executables(directory))
    if len(candidates) != 1:
        raise ValueError(f"Expected one Motion Music executable in {image.name}, found {len(candidates)}")
    return candidates[0]


def expected_hash(sidecar, filename):
    lines = [line for line in sidecar.read_text().splitlines() if line.strip()]
    if len(lines) != 1:
        raise ValueError(f"Expected one checksum in {sidecar.name}")
    match = re.fullmatch(r"([0-9a-fA-F]{64}) [ *](.+)", lines[0])
    if match is None or match[2] != filename:
        raise ValueError(f"Unexpected checksum filename in {sidecar.name}")
    return match[1].lower()


def regular_file(path):
    if path.is_symlink() or not path.is_file() or path.stat().st_size == 0:
        raise ValueError(f"Expected a nonempty regular file: {path}")


def prepare(input_directory, output_directory, notices, guide):
    sevenzip = shutil.which("7zz") or shutil.which("7z")
    if not sevenzip:
        raise ValueError("7zip is required (install Ubuntu's 7zip package)")
    license_file = Path(__file__).resolve().parents[2] / "LICENSE"
    for path in (notices, guide, license_file):
        regular_file(path)
    output_directory.parent.mkdir(parents=True, exist_ok=True)
    if output_directory.exists() and any(output_directory.iterdir()):
        raise ValueError("Output directory must be empty; existing release files are never overwritten")
    with tempfile.TemporaryDirectory(prefix="motion-mac-downloads-", dir=output_directory.parent) as temporary:
        work = Path(temporary)
        staged = work / "release"
        staged.mkdir()
        for architecture in ARCHITECTURES:
            filename = f"Motion-Music-0.14.1-macos-{architecture}.dmg"
            image = input_directory / filename
            sidecar = input_directory / (filename + ".sha256")
            for path in (image, sidecar):
                regular_file(path)
            expected = expected_hash(sidecar, filename)
            if digest(image) != expected:
                raise ValueError(f"Native builder checksum mismatch: {filename}")
            extraction = work / architecture
            extraction.mkdir()
            executable = extract_executable(image, extraction, sevenzip)
            imports = inspect_macho(executable, architecture)
            print(f"Verified {architecture}: {len(imports)} Apple system imports; native DMG hash {expected}")
            shutil.copyfile(image, staged / filename)
            shutil.copyfile(sidecar, staged / sidecar.name)
            if digest(staged / filename) != expected:
                raise ValueError(f"DMG copy changed: {filename}")
            zip_name = f"Motion-Music-v12-macos-{architecture}.zip"
            with zipfile.ZipFile(staged / zip_name, "w", compression=zipfile.ZIP_DEFLATED) as archive:
                archive.write(image, filename)
                archive.write(sidecar, sidecar.name)
                archive.write(notices, "THIRD-PARTY-NOTICES-macos.txt")
                archive.write(guide, "INSTALL.md")
                archive.write(license_file, "LICENSE")
            (staged / (zip_name + ".sha256")).write_text(f"{digest(staged / zip_name)}  {zip_name}\n")
        shutil.copyfile(notices, staged / NOTICE_NAME)
        manifest = "".join(f"{digest(path)}  {path.name}\n" for path in sorted(staged.iterdir()))
        (staged / MANIFEST_NAME).write_text(manifest)
        output_directory.mkdir(exist_ok=True)
        for path in staged.iterdir():
            # Exclusive creation also protects against a concurrent publisher.
            with path.open("rb") as source, (output_directory / path.name).open("xb") as target:
                shutil.copyfileobj(source, target)
    print(f"Prepared Mac downloads in {output_directory}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--notices", type=Path, required=True)
    parser.add_argument("--guide", type=Path, required=True)
    args = parser.parse_args()
    prepare(args.input.resolve(), args.output.resolve(), args.notices.resolve(), args.guide.resolve())


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as error:
        details = error.stdout.decode(errors="replace") if error.stdout else str(error)
        print("7zip extraction failed: " + details, file=sys.stderr)
        sys.exit(1)
    except (OSError, ValueError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
