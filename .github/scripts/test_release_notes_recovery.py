import unittest
from unittest.mock import Mock

from release_notes_github import GitHubReleaseNotes
from test_compose_release_notes import FULL_CHANGELOG, REPOSITORY, SHA1, SHA2, SHA3


class ReleaseNotesRecoveryTests(unittest.TestCase):
    def client(self, respond):
        client = GitHubReleaseNotes(REPOSITORY, "test-token")
        client.request = Mock(side_effect=respond)
        return client

    def test_unmerged_and_out_of_range_prs_are_not_recovered(self):
        for merged_at, commits in ((None, [SHA2]), ("2026-10-10", [SHA3])):
            with self.subTest(merged_at=merged_at):
                def respond(path):
                    if path == "pulls/6":
                        return {
                            "number": 6, "merged_at": merged_at,
                            "title": "Fixed notes", "user": {"login": "WhoSowSee"},
                        }
                    if path.startswith("compare/"):
                        return {"commits": [{"sha": SHA2}]}
                    if path.startswith("pulls/6/commits?"):
                        return [{"sha": sha} for sha in commits]
                    self.fail(f"Unexpected request: {path}")

                client = self.client(respond)
                self.assertIsNone(client.pull_for_release(
                    6, FULL_CHANGELOG, "v6.0.2", "https://github.com",
                ))
                if merged_at is None:
                    client.request.assert_called_once_with("pulls/6")

    def test_membership_checks_all_pages_and_reuses_the_comparison(self):
        def respond(path):
            if path in ("pulls/6", "pulls/7"):
                return {
                    "number": int(path.rsplit("/", 1)[1]),
                    "merged_at": "2026-10-10",
                    "title": "Fixed notes", "user": {"login": "WhoSowSee"},
                }
            if path.startswith("compare/"):
                commits = [SHA1] * 100 if path.endswith("page=1") else [SHA2]
                return {"commits": [{"sha": sha} for sha in commits]}
            if path.startswith("pulls/6/commits?"):
                commits = [SHA3] * 100 if path.endswith("page=1") else [SHA2]
                return [{"sha": sha} for sha in commits]
            if path.startswith("pulls/7/commits?"):
                return [{"sha": SHA2}]
            self.fail(f"Unexpected request: {path}")

        client = self.client(respond)
        for number in (6, 7):
            self.assertEqual(
                client.pull_for_release(
                    number, FULL_CHANGELOG, "v6.0.2", "https://github.com",
                ),
                {"title": "Fixed notes", "author": "WhoSowSee"},
            )
        self.assertEqual(client.request.call_count, 7)
        paths = [call.args[0] for call in client.request.call_args_list]
        self.assertEqual(sum(path.startswith("compare/") for path in paths), 2)
        self.assertIn("pulls/6/commits?per_page=100&page=2", paths)

    def test_missing_or_mismatched_comparison_is_rejected(self):
        for generated in (
            "## What's Changed",
            FULL_CHANGELOG.replace("WhoSowSee/mdv", "other/project"),
            FULL_CHANGELOG.replace("...v6.0.2", "...v6.0.3"),
        ):
            with self.subTest(generated=generated):
                client = self.client(lambda path: self.fail(path))
                with self.assertRaises(ValueError):
                    client._release_commits(generated, "v6.0.2", "https://github.com")
                client.request.assert_not_called()

    def test_comparison_supports_encoded_tags_and_enterprise_urls(self):
        generated = (
            "**Full Changelog**: https://git.example/team/mdv/"
            "compare/release%2Fold...release%2Fnew"
        )
        client = GitHubReleaseNotes("team/mdv", "test-token")
        client.request = Mock(return_value={"commits": [{"sha": SHA2}]})
        self.assertEqual(
            client._release_commits(generated, "release/new", "https://git.example/"),
            {SHA2},
        )
        client.request.assert_called_once_with(
            "compare/release%2Fold...release%2Fnew?per_page=100&page=1"
        )


if __name__ == "__main__":
    unittest.main()
