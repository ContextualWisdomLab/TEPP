"""Generate and validate fail-closed release SBOM and provenance evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
import tomllib
from pathlib import Path
from typing import Any, Iterable, Mapping, MutableMapping, Sequence


REQUIRED_SBOM_KEYS = (
    "bomFormat",
    "specVersion",
    "serialNumber",
    "version",
    "metadata",
    "components",
)
REQUIRED_PROVENANCE_KEYS = (
    "schema_version",
    "evidence_kind",
    "git_commit",
    "cargo_lock_sha256",
    "sbom_sha256",
    "component_count",
    "workspace_crate_count",
    "workspace_crates",
)


def sha256_bytes(payload: bytes) -> str:
    """Return the lowercase hex SHA-256 digest of *payload*."""

    return hashlib.sha256(payload).hexdigest()


def sha256_text(text: str) -> str:
    """Return the lowercase hex SHA-256 digest of UTF-8 *text*."""

    return sha256_bytes(text.encode("utf-8"))


def sha256_file(path: Path) -> str:
    """Return the lowercase hex SHA-256 digest of *path* contents."""

    return sha256_bytes(path.read_bytes())


def load_cargo_lock(path: Path) -> Mapping[str, Any]:
    """Load and validate a Cargo.lock document."""

    try:
        payload = tomllib.loads(path.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as error:
        raise ValueError(f"Cargo.lock is not valid TOML: {error}") from error
    packages = payload.get("package")
    if not isinstance(packages, list) or not packages:
        raise ValueError("Cargo.lock must contain a non-empty package list")
    for package in packages:
        if not isinstance(package, Mapping):
            raise ValueError("Cargo.lock package entries must be tables")
        name = package.get("name")
        version = package.get("version")
        if not isinstance(name, str) or not name:
            raise ValueError("Cargo.lock package name must be a non-empty string")
        if not isinstance(version, str) or not version:
            raise ValueError("Cargo.lock package version must be a non-empty string")
    return payload


def workspace_crate_names(cargo_toml: Path) -> list[str]:
    """Return ordered workspace member crate directory names from Cargo.toml."""

    try:
        payload = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as error:
        raise ValueError(f"Cargo.toml is not valid TOML: {error}") from error
    workspace = payload.get("workspace")
    if not isinstance(workspace, Mapping):
        raise ValueError("Cargo.toml must declare a workspace table")
    members = workspace.get("members")
    if not isinstance(members, list) or not members:
        raise ValueError("Cargo.toml workspace members must be a non-empty list")
    names: list[str] = []
    for member in members:
        if not isinstance(member, str) or not member:
            raise ValueError("workspace members must be non-empty strings")
        names.append(Path(member).name)
    return names


def component_from_package(package: Mapping[str, Any]) -> dict[str, Any]:
    """Map one Cargo.lock package table into a CycloneDX component object."""

    name = package["name"]
    version = package["version"]
    component: dict[str, Any] = {
        "type": "library",
        "name": name,
        "version": version,
        "bom-ref": f"pkg:cargo/{name}@{version}",
        "purl": f"pkg:cargo/{name}@{version}",
    }
    source = package.get("source")
    if isinstance(source, str) and source:
        component["properties"] = [{"name": "cargo:source", "value": source}]
    checksum = package.get("checksum")
    if isinstance(checksum, str) and checksum:
        component["hashes"] = [{"alg": "SHA-256", "content": checksum}]
    return component


def build_sbom(
    packages: Sequence[Mapping[str, Any]],
    *,
    git_commit: str,
    cargo_lock_sha256: str,
) -> dict[str, Any]:
    """Build a CycloneDX 1.5 JSON BOM for *packages*."""

    if not git_commit or not isinstance(git_commit, str):
        raise ValueError("git_commit must be a non-empty string")
    if len(cargo_lock_sha256) != 64:
        raise ValueError("cargo_lock_sha256 must be a 64-character hex digest")
    components = [component_from_package(package) for package in packages]
    serial = f"urn:uuid:tepp-{cargo_lock_sha256[:32]}"
    return {
        "bomFormat": "CycloneDX",
        "specVersion": "1.5",
        "serialNumber": serial,
        "version": 1,
        "metadata": {
            "component": {
                "type": "application",
                "name": "TEPP",
                "version": git_commit,
            },
            "properties": [
                {"name": "tepp:git_commit", "value": git_commit},
                {"name": "tepp:cargo_lock_sha256", "value": cargo_lock_sha256},
            ],
        },
        "components": components,
    }


def build_provenance(
    *,
    git_commit: str,
    cargo_lock_sha256: str,
    sbom_sha256: str,
    component_count: int,
    workspace_crates: Sequence[str],
) -> dict[str, Any]:
    """Build the TEPP provenance evidence document for one exact head."""

    if component_count < 0:
        raise ValueError("component_count must be non-negative")
    if not workspace_crates:
        raise ValueError("workspace_crates must not be empty")
    for name in workspace_crates:
        if not isinstance(name, str) or not name:
            raise ValueError("workspace crate names must be non-empty strings")
    return {
        "schema_version": "tepp.release_provenance.v1",
        "evidence_kind": "repository_release_evidence",
        "git_commit": git_commit,
        "cargo_lock_sha256": cargo_lock_sha256,
        "sbom_sha256": sbom_sha256,
        "component_count": component_count,
        "workspace_crate_count": len(workspace_crates),
        "workspace_crates": list(workspace_crates),
    }


def write_json(path: Path, payload: Mapping[str, Any]) -> str:
    """Write canonical JSON for *payload* and return its SHA-256 digest."""

    text = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    path.write_text(text, encoding="utf-8")
    return sha256_text(text)


def write_checksums(path: Path, digests: Mapping[str, str]) -> None:
    """Write a stable SHA-256 checksums file for *digests*."""

    lines = [f"{digest}  {name}" for name, digest in sorted(digests.items())]
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def generate_evidence(
    repository_root: Path,
    output_directory: Path,
    *,
    git_commit: str,
) -> Mapping[str, str]:
    """Generate SBOM, provenance, and checksums under *output_directory*."""

    if not git_commit.strip():
        raise ValueError("git_commit must not be blank")
    cargo_lock = repository_root / "Cargo.lock"
    cargo_toml = repository_root / "Cargo.toml"
    if not cargo_lock.is_file():
        raise ValueError("Cargo.lock must exist at the repository root")
    if not cargo_toml.is_file():
        raise ValueError("Cargo.toml must exist at the repository root")

    lock_payload = load_cargo_lock(cargo_lock)
    packages = lock_payload["package"]
    assert isinstance(packages, list)
    cargo_lock_sha256 = sha256_file(cargo_lock)
    workspace_crates = workspace_crate_names(cargo_toml)
    sbom = build_sbom(
        packages,
        git_commit=git_commit,
        cargo_lock_sha256=cargo_lock_sha256,
    )
    output_directory.mkdir(parents=True, exist_ok=True)
    sbom_path = output_directory / "sbom.cdx.json"
    provenance_path = output_directory / "provenance.json"
    checksums_path = output_directory / "checksums.sha256"
    sbom_sha256 = write_json(sbom_path, sbom)
    provenance = build_provenance(
        git_commit=git_commit,
        cargo_lock_sha256=cargo_lock_sha256,
        sbom_sha256=sbom_sha256,
        component_count=len(packages),
        workspace_crates=workspace_crates,
    )
    provenance_sha256 = write_json(provenance_path, provenance)
    digests = {
        "sbom.cdx.json": sbom_sha256,
        "provenance.json": provenance_sha256,
        "Cargo.lock": cargo_lock_sha256,
    }
    write_checksums(checksums_path, digests)
    return digests


def require_mapping(payload: Any, label: str) -> MutableMapping[str, Any]:
    """Return *payload* when it is a JSON object mapping."""

    if not isinstance(payload, MutableMapping):
        raise ValueError(f"{label} must be a JSON object")
    return payload


def validate_sbom(path: Path) -> str:
    """Validate CycloneDX SBOM shape and return a success message."""

    payload = require_mapping(json.loads(path.read_text(encoding="utf-8")), "SBOM")
    for key in REQUIRED_SBOM_KEYS:
        if key not in payload:
            raise ValueError(f"SBOM is missing required key: {key}")
    if payload.get("bomFormat") != "CycloneDX":
        raise ValueError("SBOM bomFormat must be CycloneDX")
    if payload.get("specVersion") != "1.5":
        raise ValueError("SBOM specVersion must be 1.5")
    components = payload.get("components")
    if not isinstance(components, list) or not components:
        raise ValueError("SBOM components must be a non-empty list")
    for component in components:
        if not isinstance(component, Mapping):
            raise ValueError("SBOM components must be objects")
        for field in ("type", "name", "version", "purl"):
            value = component.get(field)
            if not isinstance(value, str) or not value:
                raise ValueError(f"SBOM component {field} must be a non-empty string")
    return f"SBOM validation: PASS ({len(components)} components)"


def validate_provenance(path: Path, *, expected_git_commit: str | None = None) -> str:
    """Validate provenance evidence shape and optional commit binding."""

    payload = require_mapping(
        json.loads(path.read_text(encoding="utf-8")), "provenance"
    )
    for key in REQUIRED_PROVENANCE_KEYS:
        if key not in payload:
            raise ValueError(f"provenance is missing required key: {key}")
    if payload.get("schema_version") != "tepp.release_provenance.v1":
        raise ValueError("unsupported provenance schema_version")
    if payload.get("evidence_kind") != "repository_release_evidence":
        raise ValueError("unsupported provenance evidence_kind")
    git_commit = payload.get("git_commit")
    if not isinstance(git_commit, str) or not git_commit:
        raise ValueError("provenance git_commit must be a non-empty string")
    if expected_git_commit is not None and git_commit != expected_git_commit:
        raise ValueError("provenance git_commit does not match expected commit")
    for digest_key in ("cargo_lock_sha256", "sbom_sha256"):
        digest = payload.get(digest_key)
        if not isinstance(digest, str) or len(digest) != 64:
            raise ValueError(f"provenance {digest_key} must be a 64-character digest")
    crates = payload.get("workspace_crates")
    if not isinstance(crates, list) or not crates:
        raise ValueError("provenance workspace_crates must be a non-empty list")
    if payload.get("workspace_crate_count") != len(crates):
        raise ValueError("workspace_crate_count must match workspace_crates length")
    component_count = payload.get("component_count")
    if not isinstance(component_count, int) or component_count < 1:
        raise ValueError("component_count must be a positive integer")
    return (
        f"provenance validation: PASS (commit={git_commit}, "
        f"components={component_count}, crates={len(crates)})"
    )


def validate_checksums(path: Path, evidence_directory: Path) -> str:
    """Validate checksums file digests against files in *evidence_directory*."""

    records: dict[str, str] = {}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line:
            continue
        parts = line.split()
        if len(parts) != 2:
            raise ValueError("checksums lines must be '<digest>  <name>'")
        digest, name = parts
        if len(digest) != 64:
            raise ValueError("checksum digests must be 64 hex characters")
        if name in records:
            raise ValueError(f"duplicate checksum entry for {name}")
        records[name] = digest
    if "sbom.cdx.json" not in records or "provenance.json" not in records:
        raise ValueError("checksums must include sbom.cdx.json and provenance.json")
    for name, expected in records.items():
        if name == "Cargo.lock":
            # Cargo.lock lives at the repository root, not the evidence directory.
            continue
        target = evidence_directory / name
        if not target.is_file():
            raise ValueError(f"checksum target missing: {name}")
        actual = sha256_file(target)
        if actual != expected:
            raise ValueError(f"checksum mismatch for {name}")
    return f"checksums validation: PASS ({len(records)} entries)"


def validate_evidence_bundle(
    evidence_directory: Path,
    *,
    expected_git_commit: str | None = None,
) -> list[str]:
    """Validate SBOM, provenance, and checksums for one evidence directory."""

    sbom_path = evidence_directory / "sbom.cdx.json"
    provenance_path = evidence_directory / "provenance.json"
    checksums_path = evidence_directory / "checksums.sha256"
    for path in (sbom_path, provenance_path, checksums_path):
        if not path.is_file():
            raise ValueError(f"missing evidence file: {path.name}")
    messages = [
        validate_sbom(sbom_path),
        validate_provenance(provenance_path, expected_git_commit=expected_git_commit),
        validate_checksums(checksums_path, evidence_directory),
    ]
    provenance = json.loads(provenance_path.read_text(encoding="utf-8"))
    sbom_digest = sha256_file(sbom_path)
    if provenance.get("sbom_sha256") != sbom_digest:
        raise ValueError("provenance sbom_sha256 does not match sbom.cdx.json")
    return messages


def build_parser() -> argparse.ArgumentParser:
    """Create the release-evidence CLI parser."""

    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate", help="Generate SBOM and provenance")
    generate.add_argument("--repository-root", type=Path, default=Path.cwd())
    generate.add_argument("--output-directory", type=Path, required=True)
    generate.add_argument("--git-commit", required=True)

    validate = subparsers.add_parser("validate", help="Validate an evidence bundle")
    validate.add_argument("--evidence-directory", type=Path, required=True)
    validate.add_argument("--expected-git-commit", default=None)

    return parser


def main(arguments: Iterable[str] | None = None) -> int:
    """CLI entrypoint for generate and validate subcommands."""

    parser = build_parser()
    namespace = parser.parse_args(list(arguments) if arguments is not None else None)
    try:
        if namespace.command == "generate":
            digests = generate_evidence(
                namespace.repository_root,
                namespace.output_directory,
                git_commit=namespace.git_commit,
            )
            print(
                "release evidence: PASS "
                f"(sbom={digests['sbom.cdx.json'][:12]}…, "
                f"provenance={digests['provenance.json'][:12]}…)"
            )
            return 0
        messages = validate_evidence_bundle(
            namespace.evidence_directory,
            expected_git_commit=namespace.expected_git_commit,
        )
        for message in messages:
            print(message)
        return 0
    except (OSError, json.JSONDecodeError, ValueError) as error:
        print(f"release evidence: FAIL: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())
