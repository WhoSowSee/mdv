import unittest
from unittest.mock import Mock

from compose_release_notes import compose_notes


REPOSITORY = "WhoSowSee/mdv"
ROOT = f"https://github.com/{REPOSITORY}"
SHA1 = "a" * 40
SHA2 = "b" * 40
SHA3 = "c" * 40
FULL_CHANGELOG = f"**Full Changelog**: {ROOT}/compare/v6.0.1...v6.0.2"
GENERATED = f"""## What's Changed
* Fixed HTML rendering by @WhoSowSee in {ROOT}/pull/3
* Added a theme by @contributor in {ROOT}/pull/9

## New Contributors
* @contributor made their first contribution in {ROOT}/pull/9

{FULL_CHANGELOG}
"""


def linked_change(description, sha):
    return f"- {description} ([{sha[:7]}]({ROOT}/commit/{sha}))"


class ComposeReleaseNotesTests(unittest.TestCase):
    def compose(self, custom, associations, generated=GENERATED):
        lookup = Mock(side_effect=lambda sha: associations.get(sha, []))
        notes = compose_notes(
            custom, generated, REPOSITORY, lookup,
            resolve_pull=Mock(side_effect=AssertionError("Unexpected PR metadata lookup")),
        )
        return notes, lookup

    def test_groups_details_once_and_preserves_foreign_pr_and_commit_links(self):
        first = f"{linked_change('Fixed: visible comments', SHA1)}, [{SHA2[:7]}]({ROOT}/commit/{SHA2})"
        second = linked_change("Fixed: table lines", SHA2)
        direct = linked_change("Updated: package checks", SHA3)
        custom = f"### Bug Fixes\n\n{first}\n{second}\n\n### Maintenance\n\n{direct}"
        generated = GENERATED.replace("## What's Changed", "## Fixes")
        notes, _ = self.compose(custom, {SHA1: [3], SHA2: [3]}, generated)
        self.assertEqual(notes.count("Fixed: visible comments"), 1)
        self.assertEqual(notes.count("Fixed: table lines"), 1)
        self.assertNotIn("### Bug Fixes", notes)
        self.assertIn(f"### Maintenance\n\n{direct}\n\n## What's Changed", notes)
        self.assertIn(
            f'<summary><strong>Fixed HTML rendering</strong> by @WhoSowSee in <a href="{ROOT}/pull/3">#3</a></summary>',
            notes,
        )
        self.assertIn(first, notes)
        self.assertIn(second, notes)
        self.assertIn("<details>\n<summary><strong>Fixed HTML rendering", notes)
        self.assertNotIn("<summary>Details", notes)
        self.assertNotIn("<details open>", notes)
        self.assertNotIn("### Other Changes", notes)
        self.assertNotIn("## Fixes", notes)
        self.assertIn("## What's Changed\n\n<details>", notes)
        self.assertIn(
            f"* Added a theme by @contributor in {ROOT}/pull/9\n\n## New Contributors",
            notes,
        )
        self.assertIn(
            f"## New Contributors\n* @contributor made their first contribution in {ROOT}/pull/9\n\n---\n\n{FULL_CHANGELOG}",
            notes,
        )

    def test_explicit_pr_reference_takes_priority_over_commit_lookup(self):
        for reference in ("PR #3", "(#3)", f"[#3]({ROOT}/pull/3)"):
            with self.subTest(reference=reference):
                entry = f"{linked_change('Fixed: explicit change', SHA1)} {reference}"
                notes, lookup = self.compose(f"### Fixes\n\n{entry}", {SHA1: [9]})
                self.assertNotIn("### Fixes", notes)
                self.assertIn("- Fixed: explicit change", notes)
                self.assertIn(f"[{SHA1[:7]}]({ROOT}/commit/{SHA1})", notes)
                lookup.assert_not_called()

    def test_pr_titles_are_escaped_in_the_summary(self):
        generated = GENERATED.replace(
            "Fixed HTML rendering", "Changed <theme> & *settings*"
        )
        notes, _ = self.compose(
            linked_change("Changed: settings", SHA1), {SHA1: [3]}, generated
        )
        self.assertIn("<strong>Changed &lt;theme&gt; &amp; *settings*</strong>", notes)
        self.assertNotIn("<theme>", notes)

    def test_ambiguous_entry_requires_an_explicit_pr(self):
        entry = f"{linked_change('Fixed: changes', SHA1)} [{SHA2[:7]}]({ROOT}/commit/{SHA2})"
        with self.assertRaisesRegex(ValueError, "multiple pull requests"):
            self.compose(entry, {SHA1: [3], SHA2: [9]})

    def test_multiline_details_and_remaining_prose_are_preserved(self):
        entry = f"{linked_change('Fixed: comments', SHA1)}\n  across wrapped lines.\n\n  - Includes source numbering."
        custom = f"### Fixes\n\n{entry}\n\nRelease-specific explanation.\n\n### Documentation\n\n- Updated examples."
        notes, _ = self.compose(custom, {SHA1: [3]})
        self.assertIn("### Fixes\n\nRelease-specific explanation.", notes)
        self.assertIn("### Documentation\n\n- Updated examples.", notes)
        self.assertIn(
            "  across wrapped lines.\n\n  - Includes source numbering.", notes
        )

    def test_references_outside_this_release_remain_in_custom_notes(self):
        entry = linked_change("Fixed: direct change", SHA1)
        notes, _ = self.compose(f"### Fixes\n\n{entry}", {SHA1: [42]})
        self.assertIn(f"### Fixes\n\n{entry}\n\n## What's Changed", notes)

    def test_non_pr_references_do_not_override_commit_associations(self):
        for reference in ("issue #9", "[#6](https://github.com/other/project/pull/6)"):
            with self.subTest(reference=reference):
                entry = linked_change(f"Fixed: {reference}", SHA1)
                notes, lookup = self.compose(entry, {SHA1: [3]})
                self.assertIn(
                    f'<summary><strong>Fixed HTML rendering</strong> by @WhoSowSee in <a href="{ROOT}/pull/3">#3</a></summary>\n\n{entry}',
                    notes,
                )
                lookup.assert_called_once_with(SHA1)

    def test_code_examples_do_not_create_pr_associations(self):
        for fence in ("```", "~~~"):
            with self.subTest(fence=fence):
                example = f"  {fence}markdown\n## Empty example heading\n## Another example heading\n{linked_change('Example', SHA1)} PR #6\n  {fence}"
                entry = linked_change("Fixed: literal `PR #6` text", SHA2)
                entry += (
                    f"\n\n  {fence}text\n  PR #6\n  {ROOT}/commit/{SHA1}\n  {fence}"
                )
                notes, lookup = self.compose(f"{example}\n\n{entry}", {SHA2: [3]})
                self.assertIn(example, notes)
                self.assertIn("- Fixed: literal `PR #6` text", notes)
                lookup.assert_called_once_with(SHA2)

    def test_all_moved_items_leave_no_empty_headings_or_extra_separator(self):
        entry = linked_change("Fixed: comments", SHA1)
        notes, _ = self.compose(f"### Fixes\n\n{entry}", {SHA1: [3]})
        self.assertTrue(notes.startswith("## What's Changed"))
        self.assertEqual(notes.splitlines().count("---"), 1)
        self.assertEqual(notes.count("<details>"), 1)
        self.assertNotIn("<details open", notes)

    def test_breaking_changes_group_multi_item_prs_without_opening_details(self):
        first = linked_change("Changed: renamed theme options", SHA1)
        second = linked_change("Fixed: table lines", SHA2)
        for custom, generated in (
            (
                f"### Breaking Changes\n\n#### Theme configuration\n\n{first}\n\n### Bug Fixes\n\n{second}",
                GENERATED,
            ),
            (
                f"### Bug Fixes\n\n{first}\n{second}",
                GENERATED.replace("## What's Changed", "## Breaking Changes"),
            ),
        ):
            with self.subTest(custom=custom, generated=generated):
                notes, _ = self.compose(custom, {SHA1: [3], SHA2: [3]}, generated)
                self.assertIn("<details>\n<summary><strong>Fixed HTML rendering", notes)
                self.assertNotIn("<details open", notes)
                self.assertTrue(
                    notes.startswith("## What's Changed\n\n### Breaking Changes\n\n")
                )
                self.assertIn(first, notes)
                self.assertIn(second, notes)

    def test_breaking_and_ordinary_prs_are_grouped_without_duplication(self):
        breaking = linked_change("Changed: theme options", SHA1)
        first = linked_change("Added: theme colors", SHA2)
        second = linked_change("Added: theme examples", SHA3)
        custom = (
            f"### Breaking Changes\n\n{breaking}\n\n### Features\n\n{first}\n{second}"
        )
        generated = f"""## What's Changed
* Added a theme by @contributor in {ROOT}/pull/9
* Fixed HTML rendering by @WhoSowSee in {ROOT}/pull/3

## New Contributors
* @contributor made their first contribution in {ROOT}/pull/9
"""
        notes, _ = self.compose(custom, {SHA1: [3], SHA2: [9], SHA3: [9]}, generated)
        breaking_group, ordinary_group = notes.split("### Other Changes", 1)
        self.assertIn("### Breaking Changes", breaking_group)
        self.assertIn(f'<a href="{ROOT}/pull/3">#3</a>', breaking_group)
        self.assertIn(breaking, breaking_group)
        self.assertIn("<details>", breaking_group)
        self.assertIn(f'<a href="{ROOT}/pull/9">#9</a>', ordinary_group)
        self.assertNotIn(f'<a href="{ROOT}/pull/3">#3</a>', ordinary_group)
        self.assertNotIn(f'<a href="{ROOT}/pull/9">#9</a>', breaking_group)
        self.assertEqual(notes.count("<details>"), 2)
        self.assertNotIn("<details open", notes)
        self.assertNotIn("---", notes)

    def test_comparison_separator_follows_prs_without_new_contributors(self):
        generated = GENERATED.split("\n\n## New Contributors")[0]
        generated += f"\n\n{FULL_CHANGELOG}"
        entry = linked_change("Fixed: comments", SHA1)
        notes, _ = self.compose(entry, {SHA1: [3]}, generated)
        self.assertTrue(notes.startswith("## What's Changed"))
        self.assertTrue(
            notes.endswith(
                f"* Added a theme by @contributor in {ROOT}/pull/9\n\n---\n\n{FULL_CHANGELOG}\n"
            )
        )
        self.assertEqual(notes.splitlines().count("---"), 1)

    def test_no_generated_prs_leave_metadata_without_empty_groups(self):
        custom = linked_change("Fixed: ordinary commit", SHA1) + "  "
        for generated in (FULL_CHANGELOG, f"---\n\n{FULL_CHANGELOG}"):
            with self.subTest(generated=generated):
                notes, lookup = self.compose(custom + "\n", {SHA1: []}, generated)
                self.assertEqual(notes, f"{custom}\n\n---\n\n{FULL_CHANGELOG}\n")
                lookup.assert_not_called()

    def test_explicit_references_can_add_missing_prs_without_generated_entries(self):
        resolver = Mock(return_value={"title": "Recovered release notes", "author": "WhoSowSee"})
        lookup = Mock()
        entry = "- Fixed: release notes (PR #6)"
        notes = compose_notes(
            f"### Maintenance\n\n{entry}", FULL_CHANGELOG,
            REPOSITORY, lookup, resolve_pull=resolver,
        )
        self.assertTrue(notes.startswith("## What's Changed"))
        self.assertEqual(notes.count("<details>"), 1)
        self.assertEqual(notes.count(entry), 1)
        self.assertNotIn("### Maintenance", notes)
        resolver.assert_called_once_with(6)
        lookup.assert_not_called()

    def test_unrecognized_generated_pr_format_is_rejected(self):
        generated = (
            f"## What's Changed\n* Fixed HTML by @WhoSowSee ([#3]({ROOT}/pull/3))"
        )
        with self.assertRaisesRegex(ValueError, "unrecognized pull-request entry"):
            self.compose(linked_change("Fixed: comments", SHA1), {SHA1: [3]}, generated)


if __name__ == "__main__":
    unittest.main()
