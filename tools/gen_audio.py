#!/usr/bin/env python3
"""Generate the bundled audio asset stubs and synthesized SFX.

Per DESIGN.md §7 and §20:
- letter/<L>.wav and word/<WORD>.wav are 80ms silence stubs. The user
  records over them with a Hungarian native voice; the in-app codepath
  works either way (silent stubs are inaudible, real recordings play).
- sfx/snap.wav and sfx/chime.wav are synthesized in-script. They are
  the only files we actually want to ship as audible v1 SFX.

Run from the repo root: `python3 tools/gen_audio.py`. Idempotent.
"""

from __future__ import annotations

import json
import math
import os
import random
import struct
import wave
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
OUT = REPO_ROOT / "assets" / "audio"
WORDS_JSON = REPO_ROOT / "assets" / "words.json"

SAMPLE_RATE = 16000  # Hz; tiny + plenty for kid SFX.
SILENCE_MS = 80      # one stub length for letters and words.

LETTERS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"


def write_wav(path: Path, samples: list[float]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    pcm = bytearray()
    for s in samples:
        v = int(max(-32768.0, min(32767.0, s)))
        pcm += struct.pack("<h", v)
    with wave.open(str(path), "wb") as wf:
        wf.setnchannels(1)
        wf.setsampwidth(2)
        wf.setframerate(SAMPLE_RATE)
        wf.writeframes(bytes(pcm))


def silence(ms: int) -> list[float]:
    return [0.0] * int(SAMPLE_RATE * ms / 1000)


def tone(freq: float, ms: int, decay: float = 3.0, amp: float = 0.35) -> list[float]:
    n = int(SAMPLE_RATE * ms / 1000)
    return [
        amp * 32767.0 * math.sin(2.0 * math.pi * freq * i / SAMPLE_RATE)
        * math.exp(-decay * i / n)
        for i in range(n)
    ]


def mix(*tracks: list[float]) -> list[float]:
    n = max(len(t) for t in tracks)
    out = [0.0] * n
    for t in tracks:
        for i, v in enumerate(t):
            out[i] += v
    return out


def offset(track: list[float], ms: int) -> list[float]:
    return [0.0] * int(SAMPLE_RATE * ms / 1000) + list(track)


def snap() -> list[float]:
    """A short low-amplitude noise burst with quick exponential decay."""
    rng = random.Random(0xC117)  # deterministic snap
    n = int(SAMPLE_RATE * 60 / 1000)  # 60ms
    return [
        0.18 * 32767.0 * (rng.random() * 2.0 - 1.0) * math.exp(-12.0 * i / n)
        for i in range(n)
    ]


def chime() -> list[float]:
    """C5–E5–G5 arpeggio with overlap and exponential decay (~600ms)."""
    a = tone(523.25, 250, decay=3.0, amp=0.30)
    b = offset(tone(659.25, 250, decay=3.0, amp=0.28), 100)
    c = offset(tone(783.99, 320, decay=3.0, amp=0.26), 200)
    return mix(a, b, c)


def main() -> int:
    words = json.loads(WORDS_JSON.read_text(encoding="utf-8"))

    for letter in LETTERS:
        write_wav(OUT / "letter" / f"{letter}.wav", silence(SILENCE_MS))

    for entry in words:
        write_wav(OUT / "word" / f"{entry['word']}.wav", silence(SILENCE_MS))

    write_wav(OUT / "sfx" / "snap.wav", snap())
    write_wav(OUT / "sfx" / "chime.wav", chime())

    letters_count = len(LETTERS)
    words_count = len(words)
    total_files = letters_count + words_count + 2
    print(
        f"Wrote {total_files} files: {letters_count} letter stubs, "
        f"{words_count} word stubs, snap.wav, chime.wav -> {OUT}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
