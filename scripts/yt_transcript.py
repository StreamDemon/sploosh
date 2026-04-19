#!/usr/bin/env python3
"""Fetch a YouTube transcript and print it to stdout.

Usage:
    python3 scripts/yt_transcript.py <url-or-video-id> [--lang en] [--json] [--with-timestamps]

Requires: pip install youtube-transcript-api
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from urllib.parse import parse_qs, urlparse

VIDEO_ID_RE = re.compile(r"^[A-Za-z0-9_-]{11}$")


def extract_video_id(value: str) -> str:
    if VIDEO_ID_RE.match(value):
        return value

    parsed = urlparse(value)
    host = parsed.hostname or ""

    if host.endswith("youtu.be"):
        candidate = parsed.path.lstrip("/")
        if VIDEO_ID_RE.match(candidate):
            return candidate

    if "youtube.com" in host:
        if parsed.path == "/watch":
            v = parse_qs(parsed.query).get("v", [""])[0]
            if VIDEO_ID_RE.match(v):
                return v
        for prefix in ("/embed/", "/shorts/", "/live/", "/v/"):
            if parsed.path.startswith(prefix):
                candidate = parsed.path[len(prefix):].split("/", 1)[0]
                if VIDEO_ID_RE.match(candidate):
                    return candidate

    raise ValueError(f"Could not extract a video ID from: {value!r}")


def format_timestamp(seconds: float) -> str:
    total = int(seconds)
    h, rem = divmod(total, 3600)
    m, s = divmod(rem, 60)
    return f"{h:02d}:{m:02d}:{s:02d}" if h else f"{m:02d}:{s:02d}"


def main() -> int:
    parser = argparse.ArgumentParser(description="Fetch a YouTube transcript.")
    parser.add_argument("video", help="YouTube URL or 11-char video ID")
    parser.add_argument("--lang", default="en", help="Preferred language code (default: en)")
    parser.add_argument("--json", action="store_true", help="Emit raw snippets as JSON")
    parser.add_argument("--with-timestamps", action="store_true", help="Prefix each line with its start time")
    args = parser.parse_args()

    try:
        from youtube_transcript_api import YouTubeTranscriptApi
    except ImportError:
        print("error: youtube-transcript-api is not installed.\n"
              "  pip install youtube-transcript-api", file=sys.stderr)
        return 2

    try:
        video_id = extract_video_id(args.video)
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    api = YouTubeTranscriptApi()
    try:
        transcript = api.fetch(video_id, languages=[args.lang])
    except Exception as exc:
        print(f"error: failed to fetch transcript for {video_id}: {exc}", file=sys.stderr)
        return 1

    snippets = [
        {"start": s.start, "duration": s.duration, "text": s.text}
        for s in transcript.snippets
    ]

    if args.json:
        json.dump(snippets, sys.stdout, ensure_ascii=False, indent=2)
        sys.stdout.write("\n")
        return 0

    for snip in snippets:
        text = snip["text"].replace("\n", " ").strip()
        if not text:
            continue
        if args.with_timestamps:
            print(f"[{format_timestamp(snip['start'])}] {text}")
        else:
            print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
