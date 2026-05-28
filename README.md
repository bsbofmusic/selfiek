# SelfieK

SelfieK (`selfiek`) is a fast, evidence-friendly CLI for the K/Kate selfie pipeline.

It keeps the proven ChatGPT/CDP image-generation path and moves the orchestration into:

- **Rust core** for fast deterministic status, dice validation, sampling, prompt-card compilation, stock consumption, cleanup, and cdper invocation.
- **TypeScript npm CLI** for ergonomic distribution through `npm install -g selfiek`.

SelfieK intentionally does **not** replace `cdper-gpt-image`. cdper remains the image-generation adapter; SelfieK is the kitchen manager.

## Current defaults

SelfieK defaults to the existing local K pipeline paths:

```text
/home/agent/K-original
/home/agent/k-selfie-new
/home/agent/k-selfie-used
/home/agent/.hermes/scripts/k-selfie-generator/dice_config.json
/home/agent/obsidian-vault/raw/selfie-prompts
/home/agent/.local/bin/cdper-gpt-image
```

Override paths with flags or environment variables (`SELFIEK_K_ORIGINAL`, `SELFIEK_NEW_DIR`, `SELFIEK_USED_DIR`, `SELFIEK_DICE_CONFIG`, `SELFIEK_PROMPT_LIB`, `SELFIEK_CDPER_BIN`).

## CLI

```bash
selfiek --version
selfiek status --json
selfiek validate-config --json
selfiek library lint --json
selfiek library report --json
selfiek compile --json
selfiek compile --use-orderk --json
selfiek draw --json --use-templates --explain
selfiek generate --json --dry-run --use-templates
selfiek produce --json --use-templates --dry-run
selfiek produce --json --use-templates --quiet
selfiek next --json --use-templates
selfiek cleanup-used --json
```

## Version model

- `runtime_version`: 3.6.0
- `contract_version`: 3.6.0

SelfieK 3.6 upgrades prompt-library absorption: `library lint` validates the Obsidian recipe book, `library ingest` creates source/template drafts, `compile` emits `template_index.json`, `fragment_index.json`, `prompt_cards.jsonl`, `library_report.json`, and `weights.json`, and `draw/generate --use-templates` attach `prompt_card` v2 for feedback attribution. orderk is compile/report-only; production cron still consumes compiled artifacts and cdper remains the image-generation adapter.

The prompt-library layer also includes lightweight guardrails borrowed from prompt-optimizer-style tooling without adopting its product shape: raw prompts are stored as JSON evidence references instead of copied wholesale, risky role/injection-like source text is reported by `library lint`, `{{placeholder}}` variables are tracked through prompt cards, structured JSON sources expose top-level key preservation warnings, feedback templates are nudged toward visible facts, and `library report` emits machine-readable `quality_signals`. There is still no LLM prompt optimizer, UI, daemon, MCP optimize tool, or cdper rewrite step.
