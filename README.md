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
selfiek compile --json
selfiek draw --json --use-templates
selfiek generate --json --dry-run --use-templates
selfiek produce --json --use-templates --dry-run
selfiek produce --json --use-templates --quiet
selfiek next --json --use-templates
selfiek cleanup-used --json
```

## Version model

- `runtime_version`: 3.5.0
- `contract_version`: 3.5.0

SelfieK 3.5 activates the prompt-library compiler path at CLI level: `compile` creates `template_index.json` from Obsidian prompt templates, and `draw --use-templates` can attach matching prompt-card fragments. Production cron should switch only after local dry-run and live-generation smoke pass.
