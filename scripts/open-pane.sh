#!/usr/bin/env bash
# open-pane.sh <entrypoint> <placement> — summon a plugin pane beside the
# current work (herdr actions run a command; there is no declarative opener).
# Placement: right | down | left | up (left/up = right/down then swap).
set -uo pipefail

herdr_bin="${HERDR_BIN_PATH:-herdr}"
plugin_id="${HERDR_PLUGIN_ID:-herdr-norn}"
entry="${1:?entrypoint (skald|saga)}"
place="${2:-right}"

case "$place" in
  right) direction="right"; swap="" ;;
  down)  direction="down";  swap="" ;;
  left)  direction="right"; swap="left" ;;
  up)    direction="down";  swap="up" ;;
  *)     direction="right"; swap="" ;;
esac

args=(plugin pane open
  --plugin "$plugin_id"
  --entrypoint "$entry"
  --placement split
  --direction "$direction"
  --focus)

if [ -z "$swap" ]; then
  exec "$herdr_bin" "${args[@]}"
fi

out=$("$herdr_bin" "${args[@]}") || exit 1
pane_id=$(printf '%s' "$out" | sed -nE 's/.*"pane_id":"([^"]+)".*/\1/p' | head -1)
if [ -n "$pane_id" ]; then
  "$herdr_bin" pane swap --pane "$pane_id" --direction "$swap" >/dev/null
fi
