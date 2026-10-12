from contextlib import contextmanager
from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from threading import Thread
import unittest

from release_notes_github import GitHubReleaseNotes
from test_compose_release_notes import (
    FULL_CHANGELOG,
    GENERATED,
    REPOSITORY,
    ROOT,
    SHA1,
    SHA2,
    linked_change,
)


@contextmanager
def github_server(respond):
    requests = []

    class Handler(BaseHTTPRequestHandler):
        def handle_request(self):
            body = self.rfile.read(int(self.headers.get("Content-Length", 0)))
            data = json.loads(body) if body else None
            requests.append((self.command, self.path, data, self.headers))
            status, response = respond(self.command, self.path, data)
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(response).encode("utf-8"))

        do_GET = handle_request
        do_POST = handle_request

        def log_message(self, *_):
            pass

    server = HTTPServer(("127.0.0.1", 0), Handler)
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield f"http://127.0.0.1:{server.server_port}", requests
    finally:
        server.shutdown()
        server.server_close()
        thread.join()


class GitHubReleaseNotesTests(unittest.TestCase):
    def test_associations_are_paginated_and_only_merged_prs_are_returned(self):
        def respond(method, path, data):
            self.assertEqual(method, "GET")
            self.assertIsNone(data)
            if path.endswith("page=1"):
                return 200, [{"number": 3, "merged_at": "2026-10-08"}] * 100
            self.assertTrue(path.endswith("page=2"))
            return 200, [
                {"number": 9, "merged_at": "2026-10-09"},
                {"number": 42, "merged_at": None},
            ]

        with github_server(respond) as (url, requests):
            client = GitHubReleaseNotes(REPOSITORY, "test-token", url)
            self.assertEqual(client.pulls_for_commit(SHA1), {3, 9})
        self.assertEqual(len(requests), 2)

    def run_composer(self, root, api_url):
        script = Path(__file__).with_name("compose_release_notes.py")
        return subprocess.run(
            [
                sys.executable,
                script,
                root / "release-notes.md",
                "--repository",
                REPOSITORY,
                "--tag",
                "v6.0.2",
            ],
            env={
                **os.environ,
                "GITHUB_TOKEN": "test-token",
                "GITHUB_API_URL": api_url,
                "GITHUB_SERVER_URL": "https://github.com",
            },
            capture_output=True,
            text=True,
        )

    def test_cli_composes_body_and_caches_commit_lookups(self):
        def respond(method, path, data):
            if path.endswith("releases/generate-notes"):
                self.assertEqual(method, "POST")
                self.assertEqual(data, {"tag_name": "v6.0.2"})
                return 200, {"body": GENERATED}
            self.assertIn(f"commits/{SHA1}/pulls", path)
            return 200, [{"number": 3, "merged_at": "2026-10-08"}]

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first = linked_change("Fixed: comments", SHA1)
            second = linked_change("Fixed: tables", SHA1)
            original = f"### Bug Fixes\n\n{first}\n{second}\n"
            path = root / "release-notes.md"
            path.write_text(original, encoding="utf-8")
            with github_server(respond) as (url, requests):
                result = self.run_composer(root, url)
                self.assertEqual(result.returncode, 0, result.stderr)
            notes = path.read_text(encoding="utf-8")
            self.assertNotIn("### Bug Fixes", notes)
            self.assertIn(f"{first}\n{second}", notes)
            self.assertIn("<details>\n<summary><strong>Fixed HTML rendering", notes)
            self.assertIn(f'<a href="{ROOT}/pull/3">#3</a>', notes)
            self.assertEqual(len(requests), 2)
            for _, _, _, headers in requests:
                self.assertEqual(headers["Authorization"], "Bearer test-token")
                self.assertEqual(headers["X-GitHub-Api-Version"], "2026-03-10")

    def test_cli_recovers_explicit_pr_after_its_merge_commit_was_rewritten(self):
        generated = (
            f"## What's Changed\n"
            f"* Fixed headings by @WhoSowSee in {ROOT}/pull/5\n\n"
            f"{FULL_CHANGELOG}\n"
        )

        def respond(method, path, data):
            if path.endswith("releases/generate-notes"):
                return 200, {"body": generated}
            self.assertEqual(method, "GET")
            self.assertIsNone(data)
            if path.endswith("/pulls/6"):
                return 200, {
                    "number": 6,
                    "merged_at": "2026-10-10",
                    "title": "Fixed release notes",
                    "user": {"login": "WhoSowSee"},
                }
            if "/compare/v6.0.1...v6.0.2?" in path:
                return 200, {"commits": [{"sha": SHA1}, {"sha": SHA2}]}
            if "/pulls/6/commits?" in path:
                return 200, [{"sha": SHA2}]
            self.fail(f"Unexpected request: {path}")

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first = f"- Fixed: headings ([PR #5]({ROOT}/pull/5))"
            second = f"- Fixed: release notes ([PR #6]({ROOT}/pull/6))"
            third = "- Preserved: release metadata (PR #6)"
            path = root / "release-notes.md"
            path.write_text(
                f"### Bug Fixes\n\n{first}\n\n### Maintenance\n\n{second}\n{third}\n",
                encoding="utf-8",
            )
            with github_server(respond) as (url, requests):
                result = self.run_composer(root, url)
                self.assertEqual(result.returncode, 0, result.stderr)
            notes = path.read_text(encoding="utf-8")
        self.assertEqual(notes.count("<details>"), 2)
        for number in (5, 6):
            self.assertEqual(notes.count(f'<a href="{ROOT}/pull/{number}">#{number}</a>'), 1)
        for description in (first, second, third):
            self.assertEqual(notes.count(description), 1)
        self.assertNotIn("### Bug Fixes", notes)
        self.assertNotIn("### Maintenance", notes)
        self.assertEqual(notes.count(FULL_CHANGELOG), 1)
        self.assertEqual(len(requests), 4)

    def test_api_failure_stops_composition_and_preserves_the_input(self):
        original = (
            f"{linked_change('Fixed: comments', SHA1)}\n"
            "- Fixed: release notes (PR #6)\n"
        )
        pull = {
            "number": 6, "merged_at": "2026-10-10",
            "title": "Fixed release notes", "user": {"login": "WhoSowSee"},
        }
        for failure_path in (
            "releases/generate-notes", f"commits/{SHA1}/pulls",
            "/pulls/6", "/compare/", "/pulls/6/commits?",
        ):
            def respond(method, request_path, data):
                if failure_path in request_path:
                    return 403, {"message": "Forbidden"}
                if request_path.endswith("releases/generate-notes"):
                    return 200, {"body": GENERATED}
                if request_path.endswith("/pulls/6"):
                    return 200, pull
                if "/compare/" in request_path:
                    return 200, {"commits": [{"sha": SHA2}]}
                if "/pulls/6/commits?" in request_path:
                    return 200, [{"sha": SHA2}]
                if f"commits/{SHA1}/pulls" in request_path:
                    return 200, [{"number": 3, "merged_at": "2026-10-08"}]
                self.fail(f"Unexpected request: {request_path}")

            with (
                self.subTest(failure_path=failure_path),
                tempfile.TemporaryDirectory() as directory,
                github_server(respond) as (url, _),
            ):
                root = Path(directory)
                path = root / "release-notes.md"
                path.write_text(original, encoding="utf-8")
                result = self.run_composer(root, url)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("403", result.stderr)
                self.assertNotIn("test-token", result.stderr)
                self.assertEqual(path.read_text(encoding="utf-8"), original)


if __name__ == "__main__":
    unittest.main()
