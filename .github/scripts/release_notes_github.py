import json
from urllib.request import Request, urlopen


class GitHubReleaseNotes:
    def __init__(self, repository, token, api_url="https://api.github.com"):
        self.repository = repository
        self.token = token
        self.api_url = api_url.rstrip("/")

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

    def pulls_for_commit(self, sha):
        numbers = set()
        page = 1
        while True:
            pulls = self.request(f"commits/{sha}/pulls?per_page=100&page={page}")
            if not isinstance(pulls, list):
                raise ValueError(
                    "GitHub returned an invalid list of associated pull requests"
                )
            for pull in pulls:
                if pull.get("merged_at"):
                    numbers.add(pull["number"])
            if len(pulls) < 100:
                return numbers
            page += 1
