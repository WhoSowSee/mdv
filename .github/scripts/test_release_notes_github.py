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
from test_compose_release_notes import GENERATED, REPOSITORY, ROOT, SHA1, linked_change


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

    def test_api_failure_stops_composition_and_preserves_the_input(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "release-notes.md"
            original = f"{linked_change('Fixed: comments', SHA1)}\n"
            path.write_text(original, encoding="utf-8")
            for failure_path in ("releases/generate-notes", f"commits/{SHA1}/pulls"):

                def respond(method, request_path, data):
                    if failure_path in request_path:
                        return 403, {"message": "Forbidden"}
                    return 200, {"body": GENERATED}

                with (
                    self.subTest(failure_path=failure_path),
                    github_server(respond) as (url, _),
                ):
                    result = self.run_composer(root, url)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn("403", result.stderr)
                    self.assertNotIn("test-token", result.stderr)
                    self.assertEqual(path.read_text(encoding="utf-8"), original)


if __name__ == "__main__":
    unittest.main()
