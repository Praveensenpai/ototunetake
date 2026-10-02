# 🌸 ototunetake (音連影)

> **Dedicated Japanese Shadowing & Continuous Listening Player Powered Directly by Your Reviewed Anki Collection.**

[![Rust](https://img.shields.io/badge/Language-Rust-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Ratatui](https://img.shields.io/badge/TUI-ratatui.rs-pink.svg?style=flat-square)](https://ratatui.rs/)
[![Source](https://img.shields.io/badge/Source-Anki%202.1-lightgrey.svg?style=flat-square)](https://apps.ankiweb.net/)
[![Audio](https://img.shields.io/badge/Engine-Rodio%20Native-purple.svg?style=flat-square)](https://github.com/RustAudio/rodio)

---

```text
┌────────────────────────────────────────────────────────────────────────┐
│  🌸 Ototunetake ── Japanese Shadowing Player                           │
│  Status: [ ▶ PLAYING ]   [SHADOW]   Rep 1/2  •  Gap: 2.5s  •  Vol: 100% │
│  Sentence 12 of 756 (All Reviewed)  │  Today: 42 sent (28m, 84 reps)   │
├────────────────────────────────────────┬───────────────────────────────┤
│ 🎧 Japanese Shadowing Player           │ 📋 Queue  [SEQUENTIAL]        │
│                                        │                               │
│       平仮名が分かります。                │ ▶  12. 平仮名が分かります。       │
│      [ひらがながわかります。]             │    13. 私は猫が大好きです。     │
│  I know hiragana. I understand hiragana│    14. 明日は休みだよ。        │
│                                        │    15. コーヒーを飲みます。     │
│ [====================================] │    16. 映画を見に行きました。   │
│       ▶ Reference Audio: 00:02         │    17. これはいくらですか？    │
│                                        │    18. 駅はどこですか？        │
│    Deck: Ankidrone Foundation V7       │    19. 日本語を勉強しています。 │
│    Tags: jp1k  │  Status: Mature (49d) │    20. 今日は天気がいいね。    │
├────────────────────────────────────────┴───────────────────────────────┤
│ [Space] Play/Pause  [n/p] Next/Prev  [r] Replay  [m] Mode  [t] Text    │
│ [s] Shuffle  [P] Playlist  [/] Search  [?] Help  [q] Quit              │
└────────────────────────────────────────────────────────────────────────┘
```

**`ototunetake`** (音連影) is a keyboard-driven terminal audio player designed for Japanese shadowing and continuous listening. Inspired by the workflow and aesthetic of [**ototune**](https://github.com/Praveensenpai/ototune), it transforms sentences you have **already reviewed and learned in Anki** into an immediate, friction-free shadowing gym.

It **never** regenerates, downloads, or synthesizes audio. It reuses the exact native audio files already present in your local Anki collection, in read-only mode, without touching your review scheduling or intervals.

---

## ⚡ Features

### 🎧 Shadowing & Playback Engine
- 🗣️ **Structured Shadowing Loop**: Reference Audio $\longrightarrow$ Configurable Silence Gap $\longrightarrow$ Reference Replay $\longrightarrow$ Repetition Loop $\longrightarrow$ Auto-Advance.
- 🔁 **Three Purpose-Built Modes**:
  - **`Listen`**: Continuous native audio stream across sentences.
  - **`Repeat`**: Reference Audio $\longrightarrow$ Gap $\longrightarrow$ Next repetition $\longrightarrow$ Advance.
  - **`Shadow`**: Reference Audio $\longrightarrow$ Shadow Gap $\longrightarrow$ Reference Replay $\longrightarrow$ Advance.
- 👁️ **Audio-First Learning**: Toggle Japanese sentence text visibility on/off (`t`). Practice ear-first comprehension before checking kanji or translations (`f` for furigana, `e` for English).
- 🔀 **Shuffle & Navigation**: Jump between sentences (`n`/`p`), replay anytime (`r`), and shuffle (`s`).

### 📦 Seamless Anki Integration
- 🛡️ **Zero-Mutation Read-Only**: Connects to `collection.anki2` strictly in read-only mode (`?mode=ro`). Does not alter ease, intervals, card states, or templates.
- 🔍 **Dynamic Schema Introspection**: Handles custom SQLite collations (`unicase`), auto-detects sentence text vs vocab fields, and resolves `[sound:...]` paths in `collection.media/`.
- 🏷️ **Smart Playlists**: Instant filtering by **All Reviewed**, **Mature** ($\ge$ 21 days), **Young** ($<$ 21 days), **Recently Reviewed**, by **Deck**, or by **Tag**.
- 🔎 **Instant Search**: Press `/` to perform instant live fuzzy filtering across sentences, furigana, translations, and tags.

### 📊 Practice Stats & State Persistence
- 💾 **Session Resume**: Automatically saves active playlist, queue position, volume, and shuffle preference to `~/.config/ototunetake/session.json`.
- 📈 **Application-Level Stats**: Tracks sentences shadowed, daily repetition totals, and cumulative listening hours in `~/.config/ototunetake/stats.json` without modifying Anki history.

---

## 🚀 Installation

### 🪄 One-Liner (Recommended)

```bash
curl -sSL https://raw.githubusercontent.com/Praveensenpai/ototunetake/main/install.sh | bash
```

### 📦 Cargo Install

```bash
cargo install --path . --root ~/.local
```

### 🛠️ Building From Source

```bash
git clone https://github.com/Praveensenpai/ototunetake.git
cd ototunetake
cargo build --release
install -m 755 target/release/ototunetake ~/.local/bin/ototunetake
```

---

## ⌨️ Keybindings

| Key | Action |
| :--- | :--- |
| `Space` | Play / Pause reference audio & active shadowing loop |
| `n` / `Right` | Skip to next sentence |
| `p` / `Left` | Return to previous sentence |
| `r` | Replay current sentence reference audio |
| `m` / `Tab` | Cycle playback mode (`[Listen]` $\to$ `[Repeat]` $\to$ `[Shadow]`) |
| `t` | Toggle Japanese sentence text visibility (Audio-First Mode) |
| `f` | Toggle Furigana reading display |
| `e` | Toggle English translation display |
| `s` | Toggle Shuffle / Sequential playback |
| `+` / `-` | Increase / decrease volume |
| `[` / `]` | Adjust shadowing silence gap ($-0.5\text{s}$ / $+0.5\text{s}$) |
| `1`–`5` | Set repetition count per sentence |
| `P` | Open Playlist selection modal |
| `/` | Instant live sentence search |
| `?` | Show interactive help modal |
| `q` / `Ctrl+C` | Save session state and exit |

---

## 📖 CLI Options

```bash
# Launch player with auto-discovered Anki collection
ototunetake

# Launch directly into Listen or Repeat mode
ototunetake --mode listen

# Filter only mature cards
ototunetake --mature

# Filter by deck or tag
ototunetake --deck "Japanese::Sentences" --tag "sentence-mining"

# Launch in shuffle mode with Japanese text visible by default
ototunetake -s --show-text

# Specify custom Anki collection database and media directories
ototunetake -c ~/.local/share/Anki2/User\ 1/collection.anki2 -m ~/.local/share/Anki2/User\ 1/collection.media
```

---

## 📄 License

MIT License © [Praveen Senpai](https://github.com/Praveensenpai)
