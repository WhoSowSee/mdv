import os
from pathlib import Path
import subprocess
import tempfile
import unittest


CHANGELOG = """# Changelog

## [6.0.2] - 2026-10-03

### Bug Fixes

- Fixed: future-only change

## [6.0.1] - 2026-10-02

### Bug Fixes

- Fixed: package checks
- Fixed: Unix render timeouts

### Maintenance

- Updated: mdv-minus to v6.0.1

## [6.0.0] - 2026-10-02

### Breaking Changes

- Changed: color modes

### Features

- Added: source-line navigation

## [5.1.0] - 2026-08-22

### Features

- Added: older-only change
"""


class ReleaseNotesTests(unittest.TestCase):
    def generate(self, version, changelog=CHANGELOG, annotated=True):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = Path(directory.name)
        script = root / "release-notes.sh"
        script.write_text(
            Path(__file__).with_name("release-notes.sh").read_text(encoding="utf-8"),
            encoding="utf-8",
            newline="\n",
        )
        if changelog is not None:
            (root / "CHANGELOG.md").write_text(
                changelog, encoding="utf-8", newline="\n"
            )
        hooks = root / "hooks"
        hooks.mkdir()

        def git(*arguments):
            return subprocess.run(
                [
                    "git",
                    "-c",
                    "commit.gpgsign=false",
                    "-c",
                    f"core.hooksPath={hooks.as_posix()}",
                    *arguments,
                ],
                cwd=root,
                check=True,
                capture_output=True,
                text=True,
            )

        git("init", "--quiet")
        git("config", "user.name", "Release Notes Test")
        git("config", "user.email", "release-notes@example.invalid")
        git("add", ".")
        git("commit", "--quiet", "-m", "Commit fallback\n\nSecond paragraph")
        if annotated:
            git("tag", "-a", version, "-m", "Tag fallback\n\nSecond paragraph")
        else:
            git("tag", version)
        subprocess.run(
            ["bash", script.as_posix()],
            cwd=root,
            env={**os.environ, "GITHUB_REF": f"refs/tags/{version}"},
            check=True,
            capture_output=True,
            text=True,
        )
        return (root / "release-notes.md").read_text(encoding="utf-8")

    def test_601_includes_patch_notes_and_original_600_changes(self):
        notes = self.generate("v6.0.1")
        self.assertIn("## [6.0.1]", notes)
        self.assertIn("## [6.0.0]", notes)
        self.assertIn("- Fixed: package checks", notes)
        self.assertIn("- Changed: color modes", notes)
        self.assertIn("- Added: source-line navigation", notes)
        self.assertNotIn("future-only change", notes)
        self.assertNotIn("older-only change", notes)
        self.assertNotIn("Tag fallback", notes)
        self.assertLess(notes.index("## [6.0.1]"), notes.index("## [6.0.0]"))

    def test_600_keeps_single_section_and_changelog_priority(self):
        notes = self.generate("v6.0.0")
        self.assertTrue(notes.startswith("### Breaking Changes"))
        self.assertIn("- Changed: color modes", notes)
        self.assertNotIn("- Fixed: package checks", notes)
        self.assertNotIn("older-only change", notes)
        self.assertNotIn("Tag fallback", notes)

    def test_later_patch_does_not_repeat_600_or_601(self):
        notes = self.generate("6.0.2")
        self.assertIn("- Fixed: future-only change", notes)
        self.assertNotIn("- Fixed: package checks", notes)
        self.assertNotIn("- Changed: color modes", notes)

    def test_missing_changelog_preserves_annotated_tag_message(self):
        notes = self.generate("v6.0.1", changelog=None)
        self.assertEqual(notes, "Tag fallback\n\nSecond paragraph\n\n---")

    def test_missing_version_with_lightweight_tag_preserves_commit_message(self):
        notes = self.generate("v9.0.0", annotated=False)
        self.assertEqual(notes, "Commit fallback\n\nSecond paragraph\n\n---")


if __name__ == "__main__":
    unittest.main()
