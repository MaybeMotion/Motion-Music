#!/usr/bin/env python3
"""Collect original notices for the locked Intel and Apple Silicon Mac graphs.

The published v12 Windows notice is retained as a labelled common dependency
superset. Mac-only notices come from Cargo source files, their workspaces, or
the exact upstream commits recorded by Cargo. Three documented historical MIT
declarations retain their original manifests and a labelled SPDX reference;
other missing originals make generation fail.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import urllib.error
import urllib.parse
import urllib.request


TARGETS = ("aarch64-apple-darwin", "x86_64-apple-darwin")
RELEASE = "https://github.com/MaybeMotion/Motion-Music/releases/download/motion-v12"
BASE_NAME = "THIRD-PARTY-NOTICES-v12.txt"
BASE_SHA256 = "73ef9d548c07e908d1bf809a4ae41bc32d7cd04dcd9d6cda56da7ed7c2fe917c"
SPDX_MIT = "https://raw.githubusercontent.com/spdx/license-list-data/v3.27.0/text/MIT.txt"
SPDX_MIT_SHA256 = "b05785f9f18e6716bab63424b11454513b9943a222595b70411009202fc592b5"
# The exact published source archives for these historical releases contain
# MIT declarations and authors in Cargo.toml, but no standalone license file.
# Keep that evidence, and label the unchanged SPDX template as a reference.
DECLARATION_ONLY = {
    ("block", "0.1.6"): ("SSheldon/rust-block", "47178790cfc9d4a8b092051d8b413b78bd31254a"),
    ("dispatch", "0.2.0"): ("SSheldon/rust-dispatch", "82d6c7a5b75dc0c71c3f46f87bb6c16a476f7748"),
    ("malloc_buf", "0.0.6"): ("SSheldon/malloc_buf", "a7811e5f4c6f9685dd57ad9bfe34e9bf6b0ba4f9"),
}
NOTICE = re.compile(r"^(licen[cs]e|copying|notice|authors|copyright|unlicense|ofl)", re.I)
LICENSE = re.compile(r"^(licen[cs]e|copying|copyright|unlicense|ofl)", re.I)
SKIP = {".git", "target", "dist", ".cache", ".license-cache", "node_modules"}
UPSTREAM_NAMES = (
    "LICENSE", "LICENSE-MIT", "LICENSE-APACHE", "LICENSE.txt", "LICENSE.md",
    "COPYING", "COPYING.LESSER", "NOTICE", "AUTHORS",
)


class Originals:
    def __init__(self, cache, offline):
        self.cache = cache
        self.offline = offline
        cache.mkdir(parents=True, exist_ok=True)

    def fetch(self, url):
        key = hashlib.sha256(url.encode()).hexdigest()
        cached = self.cache / key
        absent = self.cache / (key + ".404")
        if cached.is_file():
            return cached.read_bytes()
        if absent.is_file():
            return None
        if self.offline:
            raise RuntimeError(f"Original not cached for offline use: {url}")
        request = urllib.request.Request(url, headers={"User-Agent": "Motion-Music-notices"})
        try:
            with urllib.request.urlopen(request, timeout=45) as response:
                data = response.read()
        except urllib.error.HTTPError as error:
            if error.code == 404:
                absent.touch()
                return None
            raise RuntimeError(f"Cannot read original {url}: HTTP {error.code}") from error
        except urllib.error.URLError as error:
            raise RuntimeError(f"Cannot read original {url}: {error.reason}") from error
        cached.write_bytes(data)
        return data


def metadata(manifest, target, offline):
    command = [
        "cargo", "metadata", "--locked", "--format-version", "1",
        "--manifest-path", str(manifest), "--filter-platform", target,
    ]
    if offline:
        command.append("--offline")
    return json.loads(subprocess.check_output(command, cwd=manifest.parent))


def reachable(graph):
    nodes = {node["id"]: node for node in graph["resolve"]["nodes"]}
    roots = [graph["resolve"]["root"]] if graph["resolve"]["root"] else graph["workspace_members"]
    pending = list(roots)
    seen = set()
    while pending:
        identifier = pending.pop()
        if identifier in seen:
            continue
        seen.add(identifier)
        pending.extend(dependency["pkg"] for dependency in nodes[identifier]["deps"])
    return seen


def local_originals(package, workspace):
    root = Path(package["manifest_path"]).parent
    result = {}
    if package.get("license_file"):
        file = root / package["license_file"]
        if file.is_file():
            result["crate/" + package["license_file"]] = file.read_bytes()
    if package.get("source") is None:
        candidates = [file for file in root.iterdir() if file.is_file()]
    else:
        candidates = []
        for directory, children, files in os.walk(root):
            children[:] = sorted(child for child in children if child not in SKIP)
            candidates.extend(Path(directory) / file for file in sorted(files))
    for file in candidates:
        if NOTICE.match(file.name):
            result["crate/" + file.relative_to(root).as_posix()] = file.read_bytes()
    if (package.get("source") or "").startswith("git+") or root.is_relative_to(workspace):
        for ancestor in (root, *root.parents):
            for file in sorted(ancestor.iterdir()):
                if file.is_file() and NOTICE.match(file.name):
                    result["workspace/" + file.name] = file.read_bytes()
            if (ancestor / ".git").exists() or ancestor == workspace:
                break
    return result


def github_repository(package):
    url = urllib.parse.urlsplit(package.get("repository") or "")
    if url.hostname != "github.com":
        raise RuntimeError(f"Missing originals require unsupported upstream: {package.get('repository')}")
    path = url.path.strip("/").removesuffix(".git")
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", path):
        raise RuntimeError(f"Invalid upstream repository: {package.get('repository')}")
    return path


def revision(package, originals, repository):
    root = Path(package["manifest_path"]).parent
    info = root / ".cargo_vcs_info.json"
    if info.is_file():
        data = json.loads(info.read_text())
        commit = data["git"]["sha1"]
        return commit, data.get("path_in_vcs", ""), "Cargo .cargo_vcs_info.json"
    # Older crates did not record a revision. Resolve their official version
    # tag, or use HEAD only after comparing every shipped Rust source file.
    key = originals.cache / ("revision-" + hashlib.sha256(package["id"].encode()).hexdigest())
    if key.is_file():
        return tuple(json.loads(key.read_text()))
    if originals.offline:
        raise RuntimeError(f"No cached original revision for {package['name']}")
    remote = "https://github.com/" + repository
    version = package["version"]
    refs = [f"refs/tags/{version}", f"refs/tags/v{version}"]
    output = subprocess.check_output(["git", "ls-remote", remote, *refs, *(ref + "^{}" for ref in refs), "HEAD"], text=True)
    found = dict(line.split()[::-1] for line in output.splitlines())
    commit = next((found[ref + "^{}"] if ref + "^{}" in found else found[ref] for ref in refs if ref in found), None)
    reason = "Official upstream version tag"
    if commit is None:
        commit = found.get("HEAD")
        source_files = sorted(root.rglob("*.rs"))
        if not commit or not source_files:
            raise RuntimeError(f"Cannot identify the original upstream revision for {package['name']}")
        for file in source_files:
            url = f"https://raw.githubusercontent.com/{repository}/{commit}/{file.relative_to(root).as_posix()}"
            source = originals.fetch(url)
            if source is None or source.replace(b"\r\n", b"\n") != file.read_bytes().replace(b"\r\n", b"\n"):
                raise RuntimeError(f"Unversioned upstream does not match the shipped crate source: {package['name']} {file.relative_to(root)}")
        reason = f"Official upstream HEAD; all {len(source_files)} shipped Rust files match exactly"
    result = (commit, "", reason)
    key.write_text(json.dumps(result))
    return result


def upstream_originals(package, originals):
    repository = github_repository(package)
    commit, path, reason = revision(package, originals, repository)
    if not re.fullmatch(r"[0-9a-f]{40}", commit):
        raise RuntimeError(f"Invalid upstream revision for {package['name']}")
    directories = [""]
    if path:
        relative = Path(path)
        directories.extend(parent.as_posix() for parent in (relative, *relative.parents) if parent.as_posix() != ".")
    for directory in directories:
        result = {}
        for name in UPSTREAM_NAMES:
            relative = "/".join(part for part in (directory, name) if part)
            url = f"https://raw.githubusercontent.com/{repository}/{commit}/{relative}"
            data = originals.fetch(url)
            if data:
                result[url] = data
        if any(LICENSE.match(Path(name).name) for name in result):
            return result, f"{reason}: https://github.com/{repository}/tree/{commit}"
    if DECLARATION_ONLY.get((package["name"], package["version"])) == (repository, commit):
        manifest_url = f"https://raw.githubusercontent.com/{repository}/{commit}/Cargo.toml"
        declaration = originals.fetch(manifest_url)
        reference = originals.fetch(SPDX_MIT)
        if declaration is None or reference is None or hashlib.sha256(reference).hexdigest() != SPDX_MIT_SHA256:
            raise RuntimeError(f"Cannot verify original declaration / SPDX reference for {package['name']}")
        print(f"{package['name']} {package['version']}: original standalone license absent; retaining its published manifest and labeled SPDX MIT reference", file=sys.stderr)
        return {
            manifest_url: declaration,
            "SPDX MIT reference wording (template placeholders are not crate copyright notices): " + SPDX_MIT: reference,
        }, (
            f"{reason}: https://github.com/{repository}/tree/{commit}\n"
            "No separate license/copyright file was published at this revision. The original manifest's license and authors follow verbatim.\n"
            "The SPDX MIT text is unchanged reference license wording, not an original crate copyright notice; no holder or year has been invented."
        )
    raise RuntimeError(f"No original license files found at pinned upstream for {package['name']}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest-path", type=Path, default=Path("Cargo.toml"))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--target", action="append", help="Repeat to override the default two Mac targets")
    parser.add_argument("--base-notices", type=Path, help="Existing published v12 full notices (checksum verified)")
    parser.add_argument("--cache-dir", type=Path)
    parser.add_argument("--offline", action="store_true")
    args = parser.parse_args()
    manifest = args.manifest_path.resolve(strict=True)
    output = args.output.resolve()
    originals = Originals(args.cache_dir or output.parent / ".license-cache", args.offline)
    base = args.base_notices.read_bytes() if args.base_notices else originals.fetch(RELEASE + "/" + BASE_NAME)
    if base is None or hashlib.sha256(base).hexdigest() != BASE_SHA256:
        raise RuntimeError("Published v12 base notices do not match their verified SHA-256")
    base_text = base.decode("utf-8-sig")
    covered = set(re.findall(r"^([^\r\n]+) ([0-9][^\r\n ]*)\r?\nLicense expression:", base_text, re.M))
    packages = {}
    workspace = None
    targets = args.target or TARGETS
    for target in targets:
        graph = metadata(manifest, target, args.offline)
        workspace = Path(graph["workspace_root"])
        identifiers = reachable(graph)
        packages.update((package["id"], package) for package in graph["packages"] if package["id"] in identifiers)
    sections = [
        "Motion Music v12: macOS third-party notices\n",
        "Targets: " + ", ".join(targets) + "\n",
        "The inventory conservatively includes reachable build/test dependencies; not every package is linked.\n",
        "Original declarations and texts are retained; alternative license choices are not replaced.\n",
        "Corresponding application and MilkDrop/MPL source (matching this locked application):\n",
        RELEASE + "/Motion-Music-v12-source.zip\n",
        RELEASE + "/Motion-Music-v12-MilkDrop-source.zip\n",
        "The accompanying Mac installation guide describes the native rebuild commands.\n",
        "\nPublished common dependency notices (Windows superset, retained verbatim):\n",
        f"Source: {RELEASE}/{BASE_NAME}\nSHA-256: {BASE_SHA256}\n\n",
        base_text,
        "\n\nMac dependency inventory and additional original notices\n",
    ]
    unresolved = []
    count = 0
    for package in sorted(packages.values(), key=lambda p: (p["name"], p["version"], p["id"])):
        sections.extend([
            "\n" + "=" * 80 + "\n",
            f"{package['name']} {package['version']}\n",
            f"License expression: {package.get('license') or '(not declared)'}\n",
            f"Source: {package.get('source') or 'application checkout'}\n",
            f"Repository: {package.get('repository') or '(not declared)'}\n",
            "Authors: " + "; ".join(package.get("authors", [])) + "\n",
        ])
        if (package["name"], package["version"]) in covered:
            sections.append("Original notices are included in the published common section above.\n")
            continue
        files = local_originals(package, workspace)
        provenance = "Original files from the locked Cargo source/workspace"
        if not any(LICENSE.match(Path(name).name) for name in files):
            try:
                upstream, provenance = upstream_originals(package, originals)
                files.update(upstream)
            except (RuntimeError, subprocess.CalledProcessError) as error:
                unresolved.append(f"{package['name']} {package['version']}: {error}")
                continue
        sections.append(provenance + "\n")
        for name, data in sorted(files.items()):
            text = data.decode("utf-8-sig")
            sections.extend([f"\n--- {name} ---\n", text, "\n"])
            count += 1
    # Retain the native vendored libraries' original notices independently of
    # which package's license file happened to appear in the common inventory.
    for package in packages.values():
        if package["name"] != "projectm-sys":
            continue
        native = Path(package["manifest_path"]).parent / "libprojectM"
        for directory, children, files in os.walk(native):
            children[:] = sorted(child for child in children if child not in SKIP)
            for name in sorted(files):
                if NOTICE.match(name):
                    file = Path(directory) / name
                    sections.extend([f"\n--- native libprojectM/{file.relative_to(native).as_posix()} ---\n", file.read_text(encoding="utf-8-sig"), "\n"])
                    count += 1
    if unresolved:
        raise RuntimeError("Missing original notices:\n" + "\n".join(unresolved))
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text("".join(sections), encoding="utf-8")
    print(f"Wrote {output}: {len(packages)} reachable packages; {count} additional original notice files")


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, subprocess.CalledProcessError, OSError, ValueError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
