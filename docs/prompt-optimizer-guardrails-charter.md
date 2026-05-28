# SelfieK prompt-optimizer guardrails charter

## Target boundary

SelfieK stays a lightweight prompt-library and inventory pipeline for K/Kate selfies. cdper stays the thin ChatGPT CDP image adapter. This change must not add a prompt optimizer service, LLM loop, UI, daemon, HTTP API, MCP optimize tool, or multi-model backend.

## Lessons absorbed

From `linshenkx/prompt-optimizer`, absorb only lightweight prompt-governance guardrails:

- raw prompt is stored as source/evidence, not copied wholesale into runtime prompt cards;
- risky raw-source text such as fenced code, role-like instructions, JSON blocks, `ignore previous`, or `negative_prompt` is detected by lint/report;
- `{{placeholder}}` variables are preserved from source/template into compiled prompt cards, or lint warns when source placeholders vanish;
- raw-prompt copy risk is reported when a fragment/template looks like a whole source prompt pasted into runtime;
- structured JSON raw prompts expose top-level key evidence, and templates may declare preserved keys;
- feedback notes should be grounded in visible facts, not empty quality slogans;
- prompt cards explain which guardrails fired, which placeholders were preserved, and which fragments/templates were selected.

## Non-goals

- No LLM prompt rewriting or auto-optimization.
- No web/desktop/UI/import service.
- No cdper prompt rewriting.
- No runtime cron scan of the full Obsidian prompt library.
- No automatic overwrite of Obsidian sources/templates.

## Rollback

Preflight rollback point:

- branch before work: `main` at `3a305c3`
- tag: `pre-steal-selfiek-20260528-134727`
- work branch: `steal/prompt-optimizer-guards-20260528-134727`

Rollback command:

```bash
git switch main
git reset --hard pre-steal-selfiek-20260528-134727
```

No runtime DB/schema migration is planned; runtime artifacts can be regenerated with `selfiek compile --json --use-orderk`.

## Verification gates

- RED tests first for lint/report/prompt-card guardrails.
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo build --release`
- `npm run build --workspace packages/selfiek`
- `selfiek` stable-entry smoke after installing updated artifact locally.
