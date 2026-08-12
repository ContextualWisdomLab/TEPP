"""Tests for release SBOM and provenance evidence contracts."""

from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
import unittest.mock as mock
from pathlib import Path

from scripts import release_evidence as release_evidence


class ReleaseEvidenceTests(unittest.TestCase):
    """Exercise generation, validation, and fail-closed error paths."""

    def write_lock(self, root: Path, packages: list[dict[str, object]]) -> Path:
        """Write a minimal Cargo.lock under *root*."""

        lines = ["version = 4", ""]
        for package in packages:
            lines.append("[[package]]")
            lines.append(f'name = "{package["name"]}"')
            lines.append(f'version = "{package["version"]}"')
            source = package.get("source")
            if isinstance(source, str):
                lines.append(f'source = "{source}"')
            checksum = package.get("checksum")
            if isinstance(checksum, str):
                lines.append(f'checksum = "{checksum}"')
            lines.append("")
        path = root / "Cargo.lock"
        path.write_text("\n".join(lines), encoding="utf-8")
        return path

    def write_workspace(self, root: Path, members: list[str]) -> Path:
        """Write a minimal workspace Cargo.toml under *root*."""

        member_lines = ",\n".join(f'  "{member}"' for member in members)
        text = f"[workspace]\nmembers = [\n{member_lines}\n]\n"
        path = root / "Cargo.toml"
        path.write_text(text, encoding="utf-8")
        return path

    def test_generate_and_validate_round_trip(self) -> None:
        """Generated evidence validates and binds digests to the git commit."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.write_lock(
                root,
                [
                    {
                        "name": "alpha",
                        "version": "1.0.0",
                        "source": "registry+https://github.com/rust-lang/crates.io-index",
                        "checksum": "a" * 64,
                    },
                    {"name": "workspace_crate", "version": "0.1.0"},
                ],
            )
            self.write_workspace(root, ["crates/workspace_crate", "crates/other"])
            output = root / "evidence"
            digests = release_evidence.generate_evidence(
                root, output, git_commit="abc123"
            )
            self.assertIn("sbom.cdx.json", digests)
            self.assertIn("provenance.json", digests)
            messages = release_evidence.validate_evidence_bundle(
                output,
                expected_git_commit="abc123",
                repository_root=root,
            )
            self.assertEqual(len(messages), 3)
            sbom = json.loads((output / "sbom.cdx.json").read_text(encoding="utf-8"))
            self.assertEqual(sbom["bomFormat"], "CycloneDX")
            self.assertTrue(str(sbom["serialNumber"]).startswith("urn:uuid:"))
            uuid_part = str(sbom["serialNumber"]).removeprefix("urn:uuid:")
            self.assertEqual(len(uuid_part), 36)
            self.assertEqual(len(sbom["components"]), 2)
            self.assertEqual(sbom["components"][0]["hashes"][0]["content"], "a" * 64)
            provenance = json.loads(
                (output / "provenance.json").read_text(encoding="utf-8")
            )
            self.assertEqual(provenance["workspace_crate_count"], 2)
            self.assertEqual(
                provenance["workspace_crates"], ["workspace_crate", "other"]
            )

    def test_load_and_workspace_fail_closed(self) -> None:
        """Malformed lockfiles and workspace manifests raise ValueError."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            bad_lock = root / "Cargo.lock"
            bad_lock.write_text("not = [toml", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "not valid TOML"):
                release_evidence.load_cargo_lock(bad_lock)
            empty_packages = root / "empty.lock"
            empty_packages.write_text("version = 4\npackage = []\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "non-empty package list"):
                release_evidence.load_cargo_lock(empty_packages)
            invalid_entry = root / "invalid.lock"
            invalid_entry.write_text(
                'version = 4\n[[package]]\nname = ""\nversion = "1"\n',
                encoding="utf-8",
            )
            with self.assertRaisesRegex(ValueError, "package name"):
                release_evidence.load_cargo_lock(invalid_entry)
            missing_version = root / "missing-version.lock"
            missing_version.write_text(
                'version = 4\n[[package]]\nname = "x"\nversion = ""\n',
                encoding="utf-8",
            )
            with self.assertRaisesRegex(ValueError, "package version"):
                release_evidence.load_cargo_lock(missing_version)
            non_table = root / "non-table.lock"
            non_table.write_text(
                'version = 4\npackage = ["nope"]\n', encoding="utf-8"
            )
            with self.assertRaisesRegex(ValueError, "must be tables"):
                release_evidence.load_cargo_lock(non_table)

            bad_toml = root / "Cargo.toml"
            bad_toml.write_text("workspace = [", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "not valid TOML"):
                release_evidence.workspace_crate_names(bad_toml)
            no_workspace = root / "no-ws.toml"
            no_workspace.write_text('name = "x"\n', encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "workspace table"):
                release_evidence.workspace_crate_names(no_workspace)
            empty_members = root / "empty-members.toml"
            empty_members.write_text(
                "[workspace]\nmembers = []\n", encoding="utf-8"
            )
            with self.assertRaisesRegex(ValueError, "non-empty list"):
                release_evidence.workspace_crate_names(empty_members)
            bad_member = root / "bad-member.toml"
            bad_member.write_text(
                "[workspace]\nmembers = [\"\"]\n", encoding="utf-8"
            )
            with self.assertRaisesRegex(ValueError, "non-empty strings"):
                release_evidence.workspace_crate_names(bad_member)

    def test_builders_and_validators_reject_invalid_inputs(self) -> None:
        """Builder and validator helpers fail closed on invalid shapes."""

        with self.assertRaisesRegex(ValueError, "non-empty string"):
            release_evidence.build_sbom([], git_commit="", cargo_lock_sha256="a" * 64)
        with self.assertRaisesRegex(ValueError, "64-character"):
            release_evidence.build_sbom(
                [{"name": "x", "version": "1"}],
                git_commit="c",
                cargo_lock_sha256="short",
            )
        with self.assertRaisesRegex(ValueError, "non-negative"):
            release_evidence.build_provenance(
                git_commit="c",
                cargo_lock_sha256="a" * 64,
                sbom_sha256="b" * 64,
                component_count=-1,
                workspace_crates=["crate"],
            )
        with self.assertRaisesRegex(ValueError, "must not be empty"):
            release_evidence.build_provenance(
                git_commit="c",
                cargo_lock_sha256="a" * 64,
                sbom_sha256="b" * 64,
                component_count=1,
                workspace_crates=[],
            )
        with self.assertRaisesRegex(ValueError, "non-empty strings"):
            release_evidence.build_provenance(
                git_commit="c",
                cargo_lock_sha256="a" * 64,
                sbom_sha256="b" * 64,
                component_count=1,
                workspace_crates=[""],
            )

        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            with self.assertRaisesRegex(ValueError, "must be a JSON object"):
                release_evidence.require_mapping([], "SBOM")
            bad_sbom = directory / "sbom.cdx.json"
            bad_sbom.write_text("[]\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "JSON object"):
                release_evidence.validate_sbom(bad_sbom)
            incomplete = directory / "incomplete.json"
            incomplete.write_text("{}\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "missing required key"):
                release_evidence.validate_sbom(incomplete)
            wrong_format = {
                "bomFormat": "SPDX",
                "specVersion": "1.5",
                "serialNumber": "x",
                "version": 1,
                "metadata": {},
                "components": [{"type": "library", "name": "a", "version": "1", "purl": "p"}],
            }
            wrong_path = directory / "wrong.json"
            wrong_path.write_text(json.dumps(wrong_format), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "CycloneDX"):
                release_evidence.validate_sbom(wrong_path)
            wrong_spec = dict(wrong_format)
            wrong_spec["bomFormat"] = "CycloneDX"
            wrong_spec["specVersion"] = "1.4"
            wrong_spec_path = directory / "spec.json"
            wrong_spec_path.write_text(json.dumps(wrong_spec), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "1.5"):
                release_evidence.validate_sbom(wrong_spec_path)
            empty_components = dict(wrong_spec)
            empty_components["specVersion"] = "1.5"
            empty_components["components"] = []
            empty_path = directory / "empty-components.json"
            empty_path.write_text(json.dumps(empty_components), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "non-empty list"):
                release_evidence.validate_sbom(empty_path)
            bad_component = dict(empty_components)
            bad_component["components"] = ["nope"]
            bad_component_path = directory / "bad-component.json"
            bad_component_path.write_text(json.dumps(bad_component), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "must be objects"):
                release_evidence.validate_sbom(bad_component_path)
            missing_field = dict(empty_components)
            missing_field["components"] = [{"type": "library", "name": "a", "version": "1"}]
            missing_field_path = directory / "missing-field.json"
            missing_field_path.write_text(json.dumps(missing_field), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "purl"):
                release_evidence.validate_sbom(missing_field_path)

            bad_provenance = directory / "provenance.json"
            bad_provenance.write_text("{}\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "missing required key"):
                release_evidence.validate_provenance(bad_provenance)
            base = {
                "schema_version": "other",
                "evidence_kind": "repository_release_evidence",
                "git_commit": "c",
                "cargo_lock_sha256": "a" * 64,
                "sbom_sha256": "b" * 64,
                "component_count": 1,
                "workspace_crate_count": 1,
                "workspace_crates": ["crate"],
            }
            other_schema = directory / "schema.json"
            other_schema.write_text(json.dumps(base), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "schema_version"):
                release_evidence.validate_provenance(other_schema)
            base["schema_version"] = "tepp.release_provenance.v1"
            base["evidence_kind"] = "other"
            other_kind = directory / "kind.json"
            other_kind.write_text(json.dumps(base), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "evidence_kind"):
                release_evidence.validate_provenance(other_kind)
            base["evidence_kind"] = "repository_release_evidence"
            base["git_commit"] = ""
            empty_commit = directory / "commit.json"
            empty_commit.write_text(json.dumps(base), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "git_commit"):
                release_evidence.validate_provenance(empty_commit)
            base["git_commit"] = "c"
            match_path = directory / "commit-match.json"
            match_path.write_text(json.dumps(base), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "does not match"):
                release_evidence.validate_provenance(
                    match_path, expected_git_commit="other"
                )
            base["cargo_lock_sha256"] = "short"
            short_digest = directory / "short.json"
            short_digest.write_text(json.dumps(base), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "cargo_lock_sha256"):
                release_evidence.validate_provenance(short_digest)
            base["cargo_lock_sha256"] = "a" * 64
            base["workspace_crates"] = []
            empty_crates = directory / "crates.json"
            empty_crates.write_text(json.dumps(base), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "workspace_crates"):
                release_evidence.validate_provenance(empty_crates)
            base["workspace_crates"] = ["crate"]
            base["workspace_crate_count"] = 2
            count_mismatch = directory / "count.json"
            count_mismatch.write_text(json.dumps(base), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "workspace_crate_count"):
                release_evidence.validate_provenance(count_mismatch)
            base["workspace_crate_count"] = 1
            base["component_count"] = 0
            zero_components = directory / "zero.json"
            zero_components.write_text(json.dumps(base), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "positive integer"):
                release_evidence.validate_provenance(zero_components)

            checksums = directory / "checksums.sha256"
            checksums.write_text("\nnot-a-valid-line\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "checksums lines"):
                release_evidence.validate_checksums(checksums, directory)
            checksums.write_text(f"{'a' * 10}  sbom.cdx.json\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "64 hex"):
                release_evidence.validate_checksums(checksums, directory)
            checksums.write_text(
                f"{'a' * 64}  sbom.cdx.json\n{'a' * 64}  sbom.cdx.json\n",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(ValueError, "duplicate checksum"):
                release_evidence.validate_checksums(checksums, directory)
            checksums.write_text(f"{'a' * 64}  only-one.json\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "must include"):
                release_evidence.validate_checksums(checksums, directory)
            missing_dir = directory / "missing-targets"
            missing_dir.mkdir()
            missing_checksums = missing_dir / "checksums.sha256"
            missing_checksums.write_text(
                f"{'a' * 64}  sbom.cdx.json\n{'b' * 64}  provenance.json\n",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(ValueError, "checksums must include Cargo.lock"):
                release_evidence.validate_checksums(missing_checksums, missing_dir)
            missing_checksums.write_text(
                f"{'a' * 64}  sbom.cdx.json\n"
                f"{'b' * 64}  provenance.json\n"
                f"{'c' * 64}  Cargo.lock\n",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(ValueError, "checksum target missing"):
                release_evidence.validate_checksums(missing_checksums, missing_dir)
            with self.assertRaisesRegex(ValueError, "checksum target missing"):
                release_evidence.validate_checksums(
                    missing_checksums,
                    missing_dir,
                    repository_root=missing_dir,
                )

    def test_generate_missing_inputs_and_cli(self) -> None:
        """Missing repository inputs and CLI paths remain usable."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output = root / "out"
            with self.assertRaisesRegex(ValueError, "blank"):
                release_evidence.generate_evidence(root, output, git_commit="  ")
            with self.assertRaisesRegex(ValueError, "Cargo.lock must exist"):
                release_evidence.generate_evidence(root, output, git_commit="c")
            self.write_lock(
                root,
                [{"name": "alpha", "version": "1.0.0"}],
            )
            with self.assertRaisesRegex(ValueError, "Cargo.toml must exist"):
                release_evidence.generate_evidence(root, output, git_commit="c")
            self.write_workspace(root, ["crates/alpha"])
            standard_output = io.StringIO()
            with contextlib.redirect_stdout(standard_output):
                code = release_evidence.main(
                    [
                        "generate",
                        "--repository-root",
                        str(root),
                        "--output-directory",
                        str(output),
                        "--git-commit",
                        "deadbeef",
                    ]
                )
            self.assertEqual(code, 0)
            self.assertIn("release evidence: PASS", standard_output.getvalue())
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(
                    release_evidence.main(
                        [
                            "validate",
                            "--evidence-directory",
                            str(output),
                            "--repository-root",
                            str(root),
                            "--expected-git-commit",
                            "deadbeef",
                        ]
                    ),
                    0,
                )
            # Tamper with SBOM digest binding.
            provenance_path = output / "provenance.json"
            payload = json.loads(provenance_path.read_text(encoding="utf-8"))
            payload["sbom_sha256"] = "0" * 64
            provenance_path.write_text(json.dumps(payload), encoding="utf-8")
            lock_digest = release_evidence.sha256_file(root / "Cargo.lock")
            # Refresh checksums so checksum validation does not fail first.
            release_evidence.write_checksums(
                output / "checksums.sha256",
                {
                    "sbom.cdx.json": release_evidence.sha256_file(output / "sbom.cdx.json"),
                    "provenance.json": release_evidence.sha256_file(provenance_path),
                    "Cargo.lock": lock_digest,
                },
            )
            with self.assertRaisesRegex(ValueError, "sbom_sha256 does not match"):
                release_evidence.validate_evidence_bundle(
                    output, repository_root=root
                )
            # Cargo.lock binding fails when provenance digest is wrong.
            payload["sbom_sha256"] = release_evidence.sha256_file(output / "sbom.cdx.json")
            payload["cargo_lock_sha256"] = "0" * 64
            provenance_path.write_text(json.dumps(payload), encoding="utf-8")
            release_evidence.write_checksums(
                output / "checksums.sha256",
                {
                    "sbom.cdx.json": release_evidence.sha256_file(output / "sbom.cdx.json"),
                    "provenance.json": release_evidence.sha256_file(provenance_path),
                    "Cargo.lock": lock_digest,
                },
            )
            with self.assertRaisesRegex(
                ValueError, "checksums Cargo.lock digest does not match provenance"
            ):
                release_evidence.validate_evidence_bundle(
                    output, repository_root=root
                )

            standard_error = io.StringIO()
            with contextlib.redirect_stderr(standard_error):
                self.assertEqual(
                    release_evidence.main(
                        [
                            "validate",
                            "--evidence-directory",
                            str(root / "missing"),
                            "--repository-root",
                            str(root),
                        ]
                    ),
                    1,
                )
            self.assertIn("FAIL", standard_error.getvalue())

            parser = release_evidence.build_parser()
            namespace = parser.parse_args(
                [
                    "generate",
                    "--output-directory",
                    str(output),
                    "--git-commit",
                    "c",
                ]
            )
            self.assertEqual(namespace.command, "generate")
            with mock.patch.object(
                release_evidence.sys,
                "argv",
                [
                    "release_evidence",
                    "validate",
                    "--evidence-directory",
                    str(output),
                    "--repository-root",
                    str(root),
                ],
            ):
                # Bundle is intentionally invalid after tamper; CLI returns 1.
                with contextlib.redirect_stderr(io.StringIO()):
                    self.assertEqual(release_evidence.main(None), 1)

            # Checksum mismatch path.
            good = root / "good"
            release_evidence.generate_evidence(root, good, git_commit="c")
            checksums = good / "checksums.sha256"
            lock_digest = release_evidence.sha256_file(root / "Cargo.lock")
            checksums.write_text(
                f"{'0' * 64}  sbom.cdx.json\n"
                f"{release_evidence.sha256_file(good / 'provenance.json')}  provenance.json\n"
                f"{lock_digest}  Cargo.lock\n",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(ValueError, "checksum mismatch"):
                release_evidence.validate_checksums(
                    checksums, good, repository_root=root
                )
            # Bundle missing files.
            empty = root / "empty-dir"
            empty.mkdir()
            with self.assertRaisesRegex(ValueError, "missing evidence file"):
                release_evidence.validate_evidence_bundle(empty, repository_root=root)
            # Missing repository Cargo.lock when root is provided.
            orphan = root / "orphan-root"
            orphan.mkdir()
            release_evidence.generate_evidence(root, orphan / "ev", git_commit="c")
            with self.assertRaisesRegex(ValueError, "checksum target missing: Cargo.lock"):
                release_evidence.validate_evidence_bundle(
                    orphan / "ev", repository_root=orphan
                )
            # Direct checksum mismatch after records parse.
            good_checksums = good / "checksums.sha256"
            with self.assertRaisesRegex(ValueError, "checksum mismatch"):
                release_evidence.validate_checksums(
                    good_checksums, good, repository_root=root
                )
            # Checksums Cargo.lock must match provenance cargo_lock_sha256.
            bound = root / "bound"
            release_evidence.generate_evidence(root, bound, git_commit="c")
            checksums = bound / "checksums.sha256"
            lines = checksums.read_text(encoding="utf-8").splitlines()
            rewritten = []
            for line in lines:
                if line.endswith(" Cargo.lock"):
                    rewritten.append(f"{'d' * 64}  Cargo.lock")
                else:
                    rewritten.append(line)
            checksums.write_text("\n".join(rewritten) + "\n", encoding="utf-8")
            with self.assertRaisesRegex(
                ValueError, "checksums Cargo.lock digest does not match provenance"
            ):
                # Skip file digest compare by omitting repository_root after
                # first failing path: provide root but tamper only checksums
                # after regenerating matching provenance would fail earlier on
                # checksum mismatch for Cargo.lock file. Use matching file digests
                # with mismatched provenance by rewriting provenance after.
                release_evidence.validate_evidence_bundle(bound, repository_root=None)


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
