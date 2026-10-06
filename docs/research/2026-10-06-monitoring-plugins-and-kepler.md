# Research — workflow-monitoring plugins and Kepler parity (2026-10-06)

The survey that motivated this repo. Sources: the herdr plugin marketplace
(1,581 GitHub repos tagged `herdr-plugin`, queried live), the dagr README and
CONTRACT, and GitKraken's Kepler product page and FAQ.

## Monitoring-relevant plugins found

| Plugin | Stars | What it does |
|---|---|---|
| furkankly/zoetrope | 1004 | Single-session live flow graph (turns, tool calls), terminal or phone |
| nelsonPires5/herdr-board | 170 | Kanban whose cards dispatch prompts to visible agent panes |
| hhdebb/herdr-radar | 148 | "Who's working, who's waiting on you", grouped by project |
| levi-qiao/herdr-agent-usage | 161 | Credential-scoped usage/context/rate-limit per agent |
| aemrebarut/herdr-dagr | 93 | Cross-agent run DAG with attempts, gates, evidence tiers |
| deimantasnork/captains-deck | 47 | Read-only flow kanban |
| eliasstravik/herdr-agent-progress | 31 | Agent-reported progress in the sidebar |
| Davidcreador/herdr-token-dashboard | 25 | Live token spend per pane |
| yankewei/herdr-focus-notify | 28 | Clickable macOS toasts on agent attention |
| tdi/herdr-worktree-setup | 27 | Per-project setup steps on worktree creation |

Estate already runs: shell-deck, herdr-projects, reviewr, sidebar, annotate,
command-palette, pluck, file-viewer (and dagr on ceres).

## Kepler parity map

Kepler (GitKraken's ADE) vs the estate stack:

| Kepler | Estate answer |
|---|---|
| Parallel agents in isolated worktrees | herdr panes + worktree helpers |
| Agent Graph (task/session/turn/tool-call, live) | dagr (cross-agent) + zoetrope (per-session) |
| Actions (task templates: prompt+model+skills) | blackwall tasks + herdr plugin actions (partial) |
| Per-branch diffs → stage/review/PR | reviewr + herdr-lazygit |
| Idea→issue→implement→merge thread | Linear + shell-deck + cortex mesh |
| SSH/WSL remote environments | `herdr --remote` |
| Mobile control | shell-deck phone deck |
| Worktree-creation automation | tdi/herdr-worktree-setup |
| Agent/kanban/list views | radar + board + herdr-projects |
| Swap agents per task | herdr is agent-agnostic by construction |

## The gap this repo closes

Nothing in the survey answers "one honest scrollback for a project with N
agents". Terminal-stream merging fails (alternate-screen repaints). norn's
answer: poll rendered text, keep the provably-new tail, merge by time.
**skald** reads that live; **saga** keeps it forever. See
`docs/engineering/ARCHITECTURE.md`.
