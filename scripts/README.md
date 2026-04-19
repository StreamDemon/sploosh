# scripts

Small developer utilities for working on Sploosh. Not part of the language or
toolchain — just helpers used during research and spec work.

## `yt_transcript.py`

Fetches a YouTube transcript and prints it to stdout. Useful for pulling in
talks/vlogs that inform language design decisions.

```sh
pip install youtube-transcript-api
python3 scripts/yt_transcript.py https://youtu.be/X5XKayn18ck
python3 scripts/yt_transcript.py X5XKayn18ck --with-timestamps
python3 scripts/yt_transcript.py X5XKayn18ck --json > transcript.json
```

Note: YouTube blocks transcript requests from most cloud-provider IPs. Run it
from a local machine if you hit `IpBlocked`.
