import json
import re
from urllib.parse import quote, unquote
from urllib.request import Request, urlopen


class GitHubReleaseNotes:
    def __init__(self, repository, token, api_url="https://api.github.com"):
        self.repository = repository
        self.token = token
        self.api_url = api_url.rstrip("/")
        self._comparison_commits = {}

    def request(self, path, data=None):
        headers = {
            "Accept": "application/vnd.github+json",
            "Authorization": f"Bearer {self.token}",
            "X-GitHub-Api-Version": "2026-03-10",
            "User-Agent": "mdv-release-notes",
        }
        payload = None
        if data is not None:
            headers["Content-Type"] = "application/json"
            payload = json.dumps(data).encode("utf-8")
        request = Request(
            f"{self.api_url}/repos/{self.repository}/{path}", payload, headers
        )
        with urlopen(request, timeout=30) as response:
            return json.load(response)

    def generate(self, tag):
        notes = self.request("releases/generate-notes", {"tag_name": tag})
        if not isinstance(notes, dict) or not isinstance(notes.get("body"), str):
            raise ValueError("GitHub returned invalid generated release notes")
        return notes["body"]

    def _items(self, path, field=None):
        page = 1
        while True:
            response = self.request(f"{path}?per_page=100&page={page}")
            if field is not None:
                if not isinstance(response, dict):
                    raise ValueError("GitHub returned an invalid API response")
                response = response.get(field)
            if not isinstance(response, list) or any(
                not isinstance(item, dict) for item in response
            ):
                raise ValueError("GitHub returned invalid API items")
            yield from response
            if len(response) < 100:
                return
            page += 1

    def _commit_shas(self, path, field=None):
        for commit in self._items(path, field):
            sha = commit.get("sha")
            if not isinstance(sha, str) or not sha:
                raise ValueError("GitHub returned an invalid commit")
            yield sha.lower()

    def _release_commits(self, generated, tag, server_url):
        root = re.escape(f"{server_url.rstrip('/')}/{self.repository}")
        match = re.search(
            rf"^\*\*Full Changelog\*\*:\s+{root}/compare/(\S+)\s*$",
            generated,
            re.MULTILINE,
        )
        if not match:
            raise ValueError("Cannot determine the release commit range")
        base, separator, head = unquote(match[1]).partition("...")
        if not separator or not base or head != tag:
            raise ValueError("Generated comparison does not match the release tag")
        comparison = f"{quote(base, safe='')}...{quote(head, safe='')}"
        if comparison not in self._comparison_commits:
            self._comparison_commits[comparison] = set(
                self._commit_shas(f"compare/{comparison}", "commits")
            )
        return self._comparison_commits[comparison]

    def pull_for_release(self, number, generated, tag, server_url):
        pull = self.request(f"pulls/{number}")
        if not isinstance(pull, dict) or pull.get("number") != number:
            raise ValueError("GitHub returned invalid pull-request metadata")
        if not pull.get("merged_at"):
            return None
        title = pull.get("title")
        user = pull.get("user")
        author = user.get("login") if isinstance(user, dict) else None
        if (
            not isinstance(title, str) or not title
            or not isinstance(author, str) or not author
        ):
            raise ValueError("GitHub returned invalid pull-request title or author")
        commits = self._release_commits(generated, tag, server_url)
        if not any(
            sha in commits for sha in self._commit_shas(f"pulls/{number}/commits")
        ):
            return None
        return {"title": title, "author": author}

    def pulls_for_commit(self, sha):
        return {
            pull["number"]
            for pull in self._items(f"commits/{sha}/pulls")
            if pull.get("merged_at")
        }
