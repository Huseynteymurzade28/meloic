#!/usr/bin/env python3
"""Download a small, public-domain (CC0) demo library for meloic screenshots.

All albums below were released by their artists under CC0 1.0 (public domain
dedication) and are hosted on the Internet Archive. The MP3s already carry
title/artist/album tags and an embedded cover, so meloic shows them as-is.

    python3 scripts/fetch_demo_library.py [output_dir]   # default: ./demo-music
    meloic ./demo-music

Only the Python standard library is needed. Roughly 70 MB is downloaded.
"""

import json
import sys
import urllib.parse
import urllib.request
from pathlib import Path

USER_AGENT = "meloic-demo-library/1.0 (+https://github.com/Huseynteymurzade28/meloic)"

# (archive.org identifier, artist, album, tracks to fetch — matched against file names)
ALBUMS = [
    ("Komiku-Poupis_incredible_adventures", "Komiku", "Poupi's Incredible Adventures!",
     ["TimeForTheWalkOfTheDay", "05Surfing", "DanceContestToTheMeowsic", "33SpaceMtv"]),
    ("Komiku-TaleOnTheLate", "Komiku", "Tale on the Late",
     ["03 The road we use", "09 Village, 2068", "15 You can't beat the machine"]),
    ("Komiku-the_binge_watchers", "Komiku", "The Binge Watchers: Score 1",
     ["11ActionEpic", "10AmbiantHope", "20ActionTechno"]),
    ("LoyaltyFreakMusic-ROLLERDISCODANCEDANCE", "Loyalty Freak Music", "Roller Disco Dance Dance",
     ["01 Roller Fever", "03 I'M ON FIRE", "04 Can't Stop My Feet"]),
    ("ROBOTDANCE", "Loyalty Freak Music", "Robot Dance!",
     ["01 Everyone is so alive", "02 High Technologic Beat Explosion", "03 After Party"]),
]

CC0 = "creativecommons.org/publicdomain/zero/1.0"


def fetch(url):
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=60) as response:
        return response.read()


def safe(name):
    return "".join(c for c in name if c.isalnum() or c in " -_&.,'!()").strip()


def main():
    out = Path(sys.argv[1] if len(sys.argv) > 1 else "demo-music")
    credits = ["# Demo library credits", "",
               "All music below is dedicated to the public domain under CC0 1.0 by its artists.", ""]

    for identifier, artist, album, wanted in ALBUMS:
        meta = json.loads(fetch(f"https://archive.org/metadata/{identifier}"))
        license_url = meta["metadata"].get("licenseurl", "")
        if CC0 not in license_url:
            print(f"skipping {identifier}: license is not CC0 ({license_url or 'none'})")
            continue

        album_dir = out / safe(artist) / safe(album)
        album_dir.mkdir(parents=True, exist_ok=True)
        print(f"{artist} — {album}")
        credits.append(f"- **{artist} — {album}**: https://archive.org/details/{identifier} ({license_url})")

        mp3s = [f for f in meta["files"] if f["name"].lower().endswith(".mp3")]
        for pattern in wanted:
            match = next((f for f in mp3s if pattern in f["name"]), None)
            if match is None:
                print(f"  ! no MP3 matching {pattern!r}")
                continue
            target = album_dir / safe(match["name"])
            if target.exists():
                print(f"  = {target.name}")
                continue
            url = f"https://archive.org/download/{identifier}/{urllib.parse.quote(match['name'])}"
            target.write_bytes(fetch(url))
            print(f"  + {target.name}")

    (out / "CREDITS.md").write_text("\n".join(credits) + "\n")
    print(f"\nDone. Run: meloic {out}")


if __name__ == "__main__":
    main()
