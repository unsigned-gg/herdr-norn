# AGENTS.md — herdr-norn

Agent-facing repo brief. Read this before editing.

## What this is

The norn project: passive, read-only capture of what a herdr session's agent
panes do, merged per project. **norn** records (supervised daemon), **skald**
shows it live (herdr pane), **saga** keeps it as dated markdown. Rust
workspace, dual MIT/Apache-2.0.

## Layout

- `crates/norn-domain` — pure types + the overlap diff. No I/O. If your change
  needs I/O here, you're in the wrong crate.
- `crates/norn-core` — the capture engine (herdr CLI shell-outs, appends).
- `crates/norn`, `crates/skald`, `crates/saga` — thin binaries over the two
  libraries.
- `herdr-plugin.toml` — one plugin package, two pane entrypoints.
- `docs/engineering/ARCHITECTURE.md` — the design and its invariants.
- `docs/research/` — the surveys decisions came from. `docs/sessions/` —
  dated session logs; add one when you do significant work.

## Rules that bind

- **Read-only against herdr.** Never send input to a pane. Capture is
  `agent list` + `agent read --source visible`, full stop.
- **The diff never guesses.** `new_lines` appends the overlap-proven tail or
  nothing. Any "smarter" heuristic must keep: identical ⇒ nothing, no overlap
  ⇒ nothing, first contact ⇒ nothing.
- **The recorder is a sidecar.** systemd user unit / LaunchAgent; never a
  pane, never a plugin startup hook (hooks are one-shot).
- **Pane content can carry secrets.** Feeds and narratives are local-only,
  never committed, never sent anywhere.
- Gates: `make check` (fmt + clippy -D warnings + test) must pass; commits
  signed; PR into `main`; no direct pushes.
