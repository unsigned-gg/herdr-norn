# Session log — 2026-10-06 bootstrap

## What happened

Operator directed, over one conversation on europa (mesh lane estate/w7:p7):

1. Forked `aemrebarut/herdr-dagr` → `unsigned-gg/herdr-dagr` (vendor-branch
   model per the herdr fork SOP: `main` mirrors upstream, `unsigned` is the
   distribution branch, set default).
2. Named the family: **norn** (parent/library — the Norns weave all threads),
   **skald** (the live reciter — merged feed), **saga** (the durable record).
3. Ruled the shape: Rust workspace, `norn-domain` + `norn-core` libraries,
   `norn`/`skald`/`saga` binaries, dual MIT/Apache-2.0, full scaffolding
   (AGENTS.md, docs/engineering, docs/research, docs/sessions, moon, Makefile).

## Decisions worth keeping

- Capture polls `agent read --source visible` and keeps only the
  overlap-proven new tail; repaints append nothing. (See ARCHITECTURE.md —
  "a feed that lies teaches the operator to stop reading it".)
- The recorder is a supervised sidecar, never a pane (ceres wedge, same day).
- One herdr plugin package ships both consumers: pane entrypoints `skald`
  and `saga watch`, actions `open-skald` / `open-saga`.

## Verification (this session)

- Gates: `cargo build`, `clippy -D warnings`, `cargo test` (12/12),
  `cargo fmt --check` — all green.
- Live on europa, estate session: `norn record` captured the omp pane
  (w7:p7) streaming real work into the feed and the saga narrative;
  `skald view` tailed it; `saga read` printed it; the plugin action opened
  a real skald pane (w7:pA) in the estate tab.
- The first build captured nothing for ~20 minutes of live work: agent TUIs
  keep a fixed input/status frame at the bottom, so prev's suffix never
  matched curr's prefix and every real scroll was dropped as a "repaint".
  Fixed by stripping the common trailing frame before the overlap diff
  (tests: `scroll_past_a_fixed_frame_appends_the_new_lines`,
  `a_frame_only_change_appends_nothing`). This is exactly why the rule is
  "run it live before you believe it".
- Also found: the estate opencode lane (w7:p1) is wedged — "Invalid model
  name passed in model=nvidia/NVIDIA-Nemotron-3-Ultra-550B"; prompts error
  instantly and nothing renders. Reported to the babysitting thread.
- Recorder still needs a real supervisor (LaunchAgent) — running under
  nohup for now; that's the operator's install step.
