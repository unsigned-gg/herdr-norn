# norn

**One honest scrollback per project, across every agent pane.**

A project with five agent panes has five scrollbacks and no story. Merging raw
terminal streams is a lie — agent TUIs repaint, and there is no linear
scrollback inside a pane. So norn works at the rendered-text level: it polls
each pane's visible screen and keeps only lines that are *provably* new.

- **norn** — the capture daemon (`norn record`). A supervised sidecar, never a
  pane. Read-only against herdr.
- **skald** — the live merged feed (`skald view`), one tagged stream per
  project, as a herdr split pane. The Norse reciter: it tells the work as it
  happens.
- **saga** — the durable narrative (`saga read/watch`): dated per-project
  markdown you can read, diff, and commit.

## Install

```sh
herdr plugin install unsigned-gg/herdr-norn     # then bind or invoke:
herdr plugin action invoke open-skald --plugin herdr-norn
herdr plugin action invoke open-saga  --plugin herdr-norn
```

Run the recorder as a supervised sidecar (systemd user unit shown):

```ini
[Service]
ExecStart=%h/.local/bin/norn record --session estate
Restart=on-failure
```

## The one rule

A feed that lies teaches the operator to stop reading it. `new_lines` appends
the overlap-proven tail or nothing: identical screen, nothing; wholesale
repaint, nothing; first contact, nothing. See
[docs/engineering/ARCHITECTURE.md](docs/engineering/ARCHITECTURE.md).

## Pairs with

[dagr](https://github.com/unsigned-gg/herdr-dagr) renders the *declared* run;
norn records the *observed* work. dagr is the plan, saga is the record.

MIT OR Apache-2.0.
