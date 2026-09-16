# LinkedIn presentation — nxm-webtomd

Career-positioning post (English). Copy/paste the version you prefer.
Attach the two visuals from `docs/assets/`:
- `before-after.png` — messy web page → clean Markdown
- `demo.gif` — live terminal conversion

---

## Version 1 — Primary (recommended)

I build developer tools that are small, fast, and secure by default. Here's a recent one.

**nxm-webtomd** turns any web article into clean Markdown — right inside an AI agent.

Modern web pages are ~90% noise: navigation, cookie banners, share widgets, tracking scripts. Feeding that to an LLM wastes tokens and — more importantly — opens the door to prompt-injection from hidden page content. So I built a tool that does the cleanup *before* the model ever sees the page.

What it does, and the engineering decisions behind it:

• **Secure by default** — strict fetch policy: request timeout, response size limit, SSRF protection (private/link-local networks are blocked), and content-type validation. Raw HTML/JavaScript is never executed or propagated, which mitigates prompt-injection.

• **Clean extraction** — a readability-style engine keeps the article body (headings, paragraphs, lists, links, code blocks) and drops the chrome. Output ships with structured YAML front-matter.

• **Zero-friction integration** — it speaks the Model Context Protocol over stdio, so it plugs into OpenCode, Claude Code, Cursor, and Kiro as a callable tool. One self-contained binary, no runtime downloads.

• **Written in Rust** — a ~9 MB static binary, dependency-light, installable in one line.

```
curl -fsSL https://raw.githubusercontent.com/dangranaz/nxm-webtomd/main/install.sh | sh
```

Fully open source (MIT / Apache-2.0). Code, docs, and a one-line installer here:
👉 https://github.com/dangranaz/nxm-webtomd

I'm always interested in conversations about AI tooling, Rust, and secure-by-design systems. If your team is building in this space, let's talk.

#Rust #AI #OpenSource #MCP #LLM #DeveloperTools #SoftwareEngineering #SecureByDesign

---

## Version 2 — Shorter / punchier

AI agents are great at reasoning over text. At *reading the web*? Not so much.

Every article you hand them is buried under menus, ads, and scripts — wasted tokens and a prompt-injection risk.

So I built **nxm-webtomd**: give it a URL, get clean Markdown back. No chrome, no JavaScript, just the content — with SSRF protection and prompt-injection mitigation built in.

A single Rust binary. Speaks the Model Context Protocol over stdio, so it drops into any agent (OpenCode, Claude Code, Cursor, Kiro). Installs in one line. Open source (MIT/Apache-2.0).

👉 https://github.com/dangranaz/nxm-webtomd

Building AI tooling or Rust systems on your team? I'd love to connect.

#Rust #AI #OpenSource #MCP #DeveloperTools

---

## Posting tips

- Lead with the first line — LinkedIn truncates after ~3 lines ("…see more"). Version 1's first two lines are written to hook before the fold.
- Attach `before-after.png` as the primary image (strong visual contrast reads well in the feed) and `demo.gif` as the second asset, or post the GIF as the first comment to boost reach.
- The one-line install command is deliberately visible: it signals "this is real and usable," not a slide-deck concept.
- End CTA is soft-professional: it invites recruiters/teams without sounding like you're job-hunting.
