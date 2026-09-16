#!/usr/bin/env bash
# Demo driver for the asciinema recording. Types commands with a human-like
# cadence, then runs a real nxm-webtomd conversion over stdio.
set -u

BIN="${NXM_BIN:-nxm-webtomd}"
URL="https://en.wikipedia.org/wiki/Markdown"
OUT="/tmp/article.md"

# --- tiny typewriter helpers ----------------------------------------------
type_cmd() {  # echo a prompt + command, char by char
  printf '\033[1;32m$\033[0m '
  s="$1"
  i=0
  while [ "$i" -lt "${#s}" ]; do
    printf '%s' "${s:$i:1}"
    i=$((i + 1))
    sleep 0.02
  done
  printf '\n'
}
pause() { sleep "${1:-0.6}"; }

clear
pause 0.4
type_cmd "# nxm-webtomd — web article → clean Markdown, inside your AI agent"
pause 0.5

type_cmd "nxm-webtomd  # start the MCP server (stdio)"
pause 0.4
printf '\033[90m…speaks JSON-RPC over stdin/stdout\033[0m\n'
pause 0.6

type_cmd "# convert a real article and save it to $OUT"
pause 0.4

# Real call: initialize + tools/call web_to_md → write file.
printf '%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
  "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"web_to_md\",\"arguments\":{\"url\":\"$URL\",\"output_path\":\"$OUT\"}}}" \
  | "$BIN" >/tmp/nxm_demo_out.json 2>/dev/null

pause 0.5
printf '\033[1;34m==>\033[0m fetched, extracted, stripped ads/JS, wrote Markdown:\n'
pause 0.4
# Show the confirmation line from the tool result.
grep -o 'Wrote [0-9]* bytes of Markdown to [^"]*' /tmp/nxm_demo_out.json | sed 's/^/    /'
pause 0.8

type_cmd "head -8 $OUT"
pause 0.3
head -8 "$OUT" | sed 's/^/    /'
pause 1.2

printf '\n\033[1;32m✓\033[0m clean front-matter + body — no navigation, no ads, no scripts.\n'
pause 1.6
