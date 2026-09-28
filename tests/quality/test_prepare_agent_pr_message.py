"""Exercise the trusted PR metadata parser fail-closed paths."""

from __future__ import annotations

import os
import stat
import sys
import tempfile
import unittest
import unittest.mock
from pathlib import Path
from types import ModuleType


def _load_parser() -> ModuleType:
    import scripts.prepare_agent_pr_message as module

    return module


class PrepareAgentPrMessageTests(unittest.TestCase):
    """Cover validation, limits, and CLI status codes for publication safety."""

    def setUp(self) -> None:
        self.parser = _load_parser()
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write(self, name: str, payload: bytes | str) -> Path:
        path = self.root / name
        if isinstance(payload, str):
            path.write_text(payload, encoding="utf-8")
        else:
            path.write_bytes(payload)
        return path

    def test_positive_limit_rejects_non_positive_values(self) -> None:
        with self.assertRaises(ValueError):
            self.parser._positive_limit(0, "max_title_bytes")
        with self.assertRaises(ValueError):
            self.parser._positive_limit(-1, "max_title_bytes")
        with self.assertRaises(ValueError):
            self.parser._positive_limit(True, "max_title_bytes")  # type: ignore[arg-type]
        self.assertEqual(self.parser._positive_limit(12, "max_title_bytes"), 12)

    def test_parse_happy_path_strips_blank_separator_and_sets_mode(self) -> None:
        source = self._write(
            "PR_MESSAGE.md",
            "feat: add temporal reasoner\n\nBody explains buyer gap and RED/GREEN evidence.\n",
        )
        title_path = self.root / "title.txt"
        body_path = self.root / "nested" / "body.md"
        title, body = self.parser.parse_pr_message(
            source,
            title_path,
            body_path,
            max_title_bytes=120,
            max_body_bytes=20000,
        )
        self.assertEqual(title, "feat: add temporal reasoner")
        self.assertIn("buyer gap", body)
        self.assertEqual(stat.S_IMODE(title_path.stat().st_mode), 0o600)
        self.assertEqual(stat.S_IMODE(body_path.stat().st_mode), 0o600)

    def test_rejects_missing_symlink_directory_invalid_utf8_and_controls(self) -> None:
        missing = self.root / "missing.md"
        with self.assertRaises(ValueError):
            self.parser.parse_pr_message(
                missing, self.root / "t", self.root / "b", max_title_bytes=20, max_body_bytes=20
            )

        directory = self.root / "dir"
        directory.mkdir()
        with self.assertRaises(ValueError):
            self.parser.parse_pr_message(
                directory, self.root / "t", self.root / "b", max_title_bytes=20, max_body_bytes=20
            )

        link = self.root / "link.md"
        target = self._write("target.md", "title\n\nbody\n")
        link.symlink_to(target)
        with self.assertRaises(ValueError):
            self.parser.parse_pr_message(
                link, self.root / "t", self.root / "b", max_title_bytes=20, max_body_bytes=20
            )

        invalid = self._write("bad.md", b"\xff\xfe title\n\nbody\n")
        with self.assertRaises(ValueError):
            self.parser.parse_pr_message(
                invalid, self.root / "t", self.root / "b", max_title_bytes=40, max_body_bytes=40
            )

        bidi = self._write("bidi.md", "title\u202e\n\nbody text\n")
        with self.assertRaises(ValueError):
            self.parser.parse_pr_message(
                bidi, self.root / "t", self.root / "b", max_title_bytes=40, max_body_bytes=40
            )

        control = self._write("ctrl.md", "title\x01\n\nbody text\n")
        with self.assertRaises(ValueError):
            self.parser.parse_pr_message(
                control, self.root / "t", self.root / "b", max_title_bytes=40, max_body_bytes=40
            )

    def test_rejects_empty_title_body_and_budget_overruns(self) -> None:
        empty_title = self._write("empty-title.md", "\n\nbody only\n")
        with self.assertRaises(ValueError):
            self.parser.parse_pr_message(
                empty_title, self.root / "t", self.root / "b", max_title_bytes=40, max_body_bytes=40
            )

        empty_body = self._write("empty-body.md", "title only\n")
        with self.assertRaises(ValueError):
            self.parser.parse_pr_message(
                empty_body, self.root / "t", self.root / "b", max_title_bytes=40, max_body_bytes=40
            )

        long_title = self._write("long-title.md", "t" * 50 + "\n\nbody\n")
        with self.assertRaises(ValueError):
            self.parser.parse_pr_message(
                long_title, self.root / "t", self.root / "b", max_title_bytes=10, max_body_bytes=40
            )

        long_body = self._write("long-body.md", "title\n\n" + ("b" * 50) + "\n")
        with self.assertRaises(ValueError):
            self.parser.parse_pr_message(
                long_body, self.root / "t", self.root / "b", max_title_bytes=40, max_body_bytes=10
            )

        oversized = self._write("oversized.md", "x" * 100)
        with self.assertRaises(ValueError):
            self.parser.parse_pr_message(
                oversized, self.root / "t", self.root / "b", max_title_bytes=5, max_body_bytes=5
            )

    def test_main_returns_success_and_failure_codes(self) -> None:
        source = self._write("ok.md", "feat: title\n\nBody for the product gap.\n")
        title_path = self.root / "title.txt"
        body_path = self.root / "body.md"
        status = self.parser.main(
            [str(source), str(title_path), str(body_path), "--max-title-bytes", "120"]
        )
        self.assertEqual(status, 0)
        self.assertTrue(title_path.is_file())

        missing = self.root / "nope.md"
        status = self.parser.main([str(missing), str(title_path), str(body_path)])
        self.assertEqual(status, 2)

    def test_crlf_and_cr_normalization(self) -> None:
        source = self._write("crlf.md", "feat: title\r\n\r\nBody line\rMore\n")
        title, body = self.parser.parse_pr_message(
            source,
            self.root / "t",
            self.root / "b",
            max_title_bytes=40,
            max_body_bytes=40,
        )
        self.assertEqual(title, "feat: title")
        self.assertIn("Body line", body)


if __name__ == "__main__":
    unittest.main()


class PrepareAgentPrMessageHardeningTests(unittest.TestCase):
    """Cover rare filesystem race and budget paths with controlled doubles."""

    def setUp(self) -> None:
        self.parser = _load_parser()
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_title_and_body_byte_budgets_after_successful_read(self) -> None:
        # Keep total payload under title+body+4 so the size gate does not fire first.
        source = self.root / "budget.md"
        source.write_text("abcdefghij\n\nbody-ok\n", encoding="utf-8")
        with self.assertRaises(ValueError):
            self.parser.parse_pr_message(
                source,
                self.root / "t",
                self.root / "b",
                max_title_bytes=5,
                max_body_bytes=40,
            )
        source.write_text("title\n\n" + ("x" * 20), encoding="utf-8")
        with self.assertRaises(ValueError):
            self.parser.parse_pr_message(
                source,
                self.root / "t",
                self.root / "b",
                max_title_bytes=40,
                max_body_bytes=5,
            )

    def test_body_without_blank_separator_line(self) -> None:
        source = self.root / "no-sep.md"
        source.write_text("feat: title\nBody continues immediately.\n", encoding="utf-8")
        title, body = self.parser.parse_pr_message(
            source,
            self.root / "t",
            self.root / "b",
            max_title_bytes=40,
            max_body_bytes=80,
        )
        self.assertEqual(title, "feat: title")
        self.assertTrue(body.startswith("Body continues"))

    def test_open_failures_and_identity_races_are_fail_closed(self) -> None:
        source = self.root / "race.md"
        source.write_text("title\n\nbody\n", encoding="utf-8")

        with unittest.mock.patch(
            "scripts.prepare_agent_pr_message.os.open", side_effect=OSError("denied")
        ):
            with self.assertRaises(ValueError):
                self.parser._read_regular_file(source, 100)

        real_fstat = os.fstat

        def fake_fstat(fd: int):  # noqa: ANN001
            info = real_fstat(fd)
            # Mutate mode so S_ISREG fails.
            return os.stat_result(
                (info.st_mode & ~stat.S_IFREG, *info[1:])
            )

        with unittest.mock.patch(
            "scripts.prepare_agent_pr_message.os.fstat", side_effect=fake_fstat
        ):
            with self.assertRaises(ValueError):
                self.parser._read_regular_file(source, 100)

        # st_dev/st_ino positions: mode,ino,dev,... actually index: st_mode=0,st_ino=1,st_dev=2
        def identity_shift2(fd: int):  # noqa: ANN001
            info = real_fstat(fd)
            values = list(info)
            values[1] = info.st_ino + 99
            return os.stat_result(values)

        with unittest.mock.patch(
            "scripts.prepare_agent_pr_message.os.fstat", side_effect=identity_shift2
        ):
            with self.assertRaises(ValueError):
                self.parser._read_regular_file(source, 100)

    def test_write_and_read_finally_close_paths(self) -> None:
        path = self.root / "out.txt"
        with unittest.mock.patch(
            "scripts.prepare_agent_pr_message.os.fdopen",
            side_effect=OSError("write-fail"),
        ):
            with self.assertRaises(OSError):
                self.parser._write_private_text(path, "value")

        source = self.root / "read-fail.md"
        source.write_text("title\n\nbody\n", encoding="utf-8")

        def boom(*args, **kwargs):  # noqa: ANN001, ANN002
            raise OSError("read-fail")

        with unittest.mock.patch(
            "scripts.prepare_agent_pr_message.os.fdopen", side_effect=boom
        ):
            with self.assertRaises(OSError):
                self.parser._read_regular_file(source, 100)

    def test_module_entrypoint_invokes_main(self) -> None:
        import runpy

        source = self.root / "entry.md"
        source.write_text("feat: entry\n\nBody text for entrypoint coverage.\n", encoding="utf-8")
        title = self.root / "t.txt"
        body = self.root / "b.md"
        with unittest.mock.patch(
            "sys.argv",
            [
                "prepare_agent_pr_message.py",
                str(source),
                str(title),
                str(body),
            ],
        ):
            loaded_module = sys.modules.pop("scripts.prepare_agent_pr_message", None)
            try:
                with self.assertRaises(SystemExit) as raised:
                    runpy.run_module(
                        "scripts.prepare_agent_pr_message", run_name="__main__"
                    )
            finally:
                if loaded_module is not None:
                    sys.modules["scripts.prepare_agent_pr_message"] = loaded_module
            self.assertEqual(raised.exception.code, 0)

    def test_read_without_nofollow_flag_when_unavailable(self) -> None:
        source = self.root / "plain.md"
        source.write_text("title\n\nbody\n", encoding="utf-8")
        real_hasattr = hasattr

        def selective_hasattr(obj, name):  # noqa: ANN001
            if obj is os and name == "O_NOFOLLOW":
                return False
            return real_hasattr(obj, name)

        with unittest.mock.patch(
            "scripts.prepare_agent_pr_message.hasattr", side_effect=selective_hasattr
        ):
            payload = self.parser._read_regular_file(source, 100)
        self.assertTrue(payload.startswith(b"title"))
