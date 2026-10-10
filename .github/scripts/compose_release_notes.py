import argparse
from collections import defaultdict
from html import escape
import os
from pathlib import Path
import re

from release_notes_github import GitHubReleaseNotes


HEADING = re.compile(r"^(#{1,6})\s+\S")
ITEM = re.compile(r"^[-*+]\s+\S")
FENCE = re.compile(r"^ {0,3}(`{3,}|~{3,})")
INLINE_CODE = re.compile(r"(`+).*?\1", re.DOTALL)
MARKDOWN_LINK = re.compile(r"\[[^\]\n]*\]\([^\n)]*\)")
PR_NUMBER = re.compile(r"\bPR\s+#([1-9]\d*)\b|\(#([1-9]\d*)\)", re.IGNORECASE)


def update_fence(line, fence):
    marker = FENCE.match(line)
    if marker:
        if fence is None:
            fence = marker[1]
        elif (
            marker[1][0] == fence[0]
            and len(marker[1]) >= len(fence)
            and not line[marker.end() :].strip()
        ):
            fence = None
    return fence, marker is not None


def update_sections(line, sections, index):
    heading = HEADING.match(line)
    if heading:
        level = len(heading[1])
        while sections and sections[-1][0] >= level:
            sections.pop()
        title = line[heading.end(1) :].strip().rstrip("#").strip()
        sections.append((level, title.casefold() == "breaking changes", index))
    return any(breaking for _, breaking, _ in sections)


def list_items(lines):
    index = 0
    fence = None
    sections = []
    while index < len(lines):
        line = lines[index]
        fence, marker = update_fence(line, fence)
        if fence is not None or marker:
            index += 1
            continue
        breaking = update_sections(line, sections, index)
        if not ITEM.match(line):
            index += 1
            continue
        end = index + 1
        while end < len(lines):
            following = lines[end]
            if (
                ITEM.match(following)
                or HEADING.match(following)
                or (FENCE.match(following) and not following.startswith(" "))
            ):
                break
            if not following.strip():
                after = end + 1
                while after < len(lines) and not lines[after].strip():
                    after += 1
                if after == len(lines) or not lines[after].startswith((" ", "\t")):
                    break
            end += 1
        yield index, end, breaking
        index = end


def without_empty_headings(lines):
    marked = []
    fence = None
    for line in lines:
        fence, marker = update_fence(line, fence)
        marked.append(
            (line, HEADING.match(line) if fence is None and not marker else None)
        )
    for index in reversed(range(len(marked))):
        _, heading = marked[index]
        if not heading:
            continue
        end = index + 1
        while end < len(marked):
            _, next_heading = marked[end]
            if next_heading and len(next_heading[1]) <= len(heading[1]):
                break
            end += 1
        if not any(
            line.strip() and not is_heading
            for line, is_heading in marked[index + 1 : end]
        ):
            del marked[index:end]
    while marked and not marked[0][0].strip():
        del marked[0]
    while marked and not marked[-1][0].strip():
        marked.pop()
    return "\n".join(line for line, _ in marked)


def reference_patterns(repository, server_url):
    root = re.escape(f"{server_url.rstrip('/')}/{repository}")
    pull = rf"{root}/pull/([1-9]\d*)\b"
    commit = rf"{root}/commit/([a-fA-F0-9]{{7,40}})\b"
    return re.compile(pull, re.IGNORECASE), re.compile(commit, re.IGNORECASE)


def associated_pull(item, patterns, eligible, lookup):
    visible = []
    fence = None
    for line in item:
        fence, marker = update_fence(line.lstrip(), fence)
        if fence is None and not marker:
            visible.append(line)
    text = INLINE_CODE.sub("", "\n".join(visible))
    pull_pattern, commit_pattern = patterns
    explicit = {int(match[1]) for match in pull_pattern.finditer(text)}
    explicit.update(
        int(match[1] or match[2])
        for match in PR_NUMBER.finditer(MARKDOWN_LINK.sub("", text))
    )
    if explicit:
        numbers = explicit
    else:
        numbers = set()
        for match in commit_pattern.finditer(text):
            numbers.update(lookup(match[1].lower()))
        numbers &= eligible
    if len(numbers) > 1:
        raise ValueError(
            "A changelog item refers to multiple pull requests; specify one PR explicitly"
        )
    return next(iter(numbers & eligible), None)


def generated_pull_pattern(repository, server_url):
    root = re.escape(f"{server_url.rstrip('/')}/{repository}")
    return re.compile(
        rf"^[-*+] (?P<title>.+) by @(?P<author>[\w\[\]-]+) "
        rf"in (?P<url>{root}/pull/(?P<number>[1-9]\d*))\s*$"
    )


def grouped_pr_entries(entries, breaking_pulls):
    if not breaking_pulls:
        return "## What's Changed\n\n" + "\n\n".join(
            entry.rstrip() for entry in entries.values()
        )
    groups = []
    for title, breaking in (("Breaking Changes", True), ("Other Changes", False)):
        blocks = [
            entry.rstrip()
            for number, entry in entries.items()
            if (number in breaking_pulls) == breaking
        ]
        if blocks:
            groups.append(f"### {title}\n\n" + "\n\n".join(blocks))
    return "## What's Changed\n\n" + "\n\n".join(groups)


def compose_notes(
    custom, generated, repository, lookup, server_url="https://github.com"
):
    custom = custom.rstrip("\r\n")
    generated_lines = generated.strip().splitlines()
    pattern = generated_pull_pattern(repository, server_url)
    patterns = reference_patterns(repository, server_url)
    pull_reference, _ = patterns
    pull_rows = {}
    generated_removed = set()
    breaking_pulls = set()
    sections = []
    contributors = False
    for index, line in enumerate(generated_lines):
        breaking = update_sections(line, sections, index)
        if HEADING.match(line):
            contributors = line.strip() == "## New Contributors"
        match = pattern.match(line)
        if match:
            number = int(match["number"])
            if number in pull_rows:
                raise ValueError(f"GitHub generated duplicate entries for PR #{number}")
            pull_rows[number] = match
            generated_removed.add(index)
            generated_removed.update(position for _, _, position in sections)
            if breaking:
                breaking_pulls.add(number)
        elif not contributors and ITEM.match(line) and pull_reference.search(line):
            raise ValueError(
                "GitHub returned an unrecognized pull-request entry format"
            )
    eligible = set(pull_rows)
    cache = {}

    def cached_lookup(sha):
        if sha not in cache:
            cache[sha] = lookup(sha)
        return cache[sha]

    lines = custom.splitlines()
    removed = set()
    details = defaultdict(list)
    if eligible:
        for start, end, breaking in list_items(lines):
            item = lines[start:end]
            number = associated_pull(item, patterns, eligible, cached_lookup)
            if number is not None:
                details[number].append("\n".join(item))
                if breaking:
                    breaking_pulls.add(number)
                while start > 0 and not lines[start - 1].strip():
                    start -= 1
                removed.update(range(start, end))
    entries = {}
    for number, match in pull_rows.items():
        items = details.get(number)
        if not items:
            entries[number] = match[0]
            continue
        summary = (
            f"<strong>{escape(match['title'])}</strong> by @{escape(match['author'])} "
            f'in <a href="{escape(match["url"])}">#{number}</a>'
        )
        descriptions = "\n".join(items)
        entries[number] = (
            f"<details>\n<summary>{summary}</summary>\n\n{descriptions}\n\n</details>"
        )
    if entries:
        position = min(generated_removed)
        generated_lines[position] = grouped_pr_entries(entries, breaking_pulls)
        generated_lines = [
            line
            for index, line in enumerate(generated_lines)
            if index == position or index not in generated_removed
        ]
    for index, line in enumerate(generated_lines):
        if line.startswith("**Full Changelog**:"):
            start = index
            while start and generated_lines[start - 1].strip() in ("", "---"):
                start -= 1
            generated_lines[start:index] = ["", "---", ""]
            break
    remaining = without_empty_headings(
        line for index, line in enumerate(lines) if index not in removed
    )
    parts = [part for part in (remaining, "\n".join(generated_lines).strip()) if part]
    return "\n\n".join(parts) + "\n"


def main():
    parser = argparse.ArgumentParser(
        description="Compose release notes with detailed PR entries."
    )
    parser.add_argument("body", type=Path)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--tag", required=True)
    args = parser.parse_args()
    if not re.fullmatch(r"[\w.-]+/[\w.-]+", args.repository):
        parser.error("--repository must be OWNER/REPO")
    token = os.environ.get("GITHUB_TOKEN")
    if not token:
        parser.error("GITHUB_TOKEN is required")
    client = GitHubReleaseNotes(
        args.repository,
        token,
        os.environ.get("GITHUB_API_URL", "https://api.github.com"),
    )
    try:
        generated = client.generate(args.tag)
        notes = compose_notes(
            args.body.read_text(encoding="utf-8"),
            generated,
            args.repository,
            client.pulls_for_commit,
            os.environ.get("GITHUB_SERVER_URL", "https://github.com"),
        )
        args.body.write_text(notes, encoding="utf-8", newline="\n")
    except (OSError, ValueError) as error:
        parser.exit(1, f"Cannot compose release notes: {error}\n")


if __name__ == "__main__":
    main()
