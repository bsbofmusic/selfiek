# SelfieK

SelfieK (`selfiek`) is a fast, evidence-friendly CLI for the K/Kate selfie pipeline.

It keeps the proven ChatGPT/CDP image-generation path and moves the orchestration into:

- **Rust core** for fast deterministic status, dice validation, sampling, prompt-card compilation, stock consumption, cleanup, and cdper invocation.
- **TypeScript npm CLI** for ergonomic distribution through `npm install -g selfiek`.

SelfieK intentionally does **not** replace `cdper-gpt-image`. cdper remains the image-generation adapter; SelfieK is the stock kitchen manager.

Boundary: **SelfieK manages the fridge; cdper handles ad-hoc cooking.** SelfieK can call cdper to produce inventory, but ad-hoc/one-off image requests should go directly to cdper instead of adding an instant lane to SelfieK.

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

Override paths with flags or environment variables (`SELFIEK_K_ORIGINAL`, `SELFIEK_NEW_DIR`, `SELFIEK_USED_DIR`, `SELFIEK_DICE_CONFIG`, `SELFIEK_PROMPT_LIB`, `SELFIEK_CDPER_BIN`). For stock-only product-line generalization, use `--profile-file` / `SELFIEK_PROFILE_FILE` plus `--profile` and `--stock-limit`.

Profile manifest example:

```yaml
schema_version: selfiek.profile.v1
id: k-selfie
paths:
  dice_config: dice_config.json
  reference_dir: /home/agent/K-original
  new_dir: /home/agent/k-selfie-new
  used_dir: /home/agent/k-selfie-used
  prompt_lib: /home/agent/obsidian-vault/raw/selfie-prompts
  runtime_dir: /home/agent/.hermes/scripts/k-selfie-generator
  cdper_bin: /home/agent/.local/bin/cdper-gpt-image
stock:
  new_limit: 100
```

Profile paths may be relative to the manifest file. Precedence is: built-in defaults < profile file < environment variables < CLI flags.

## CLI

```bash
selfiek --version
selfiek status --json
selfiek profile show --json
selfiek profile validate --file ./profile.yml --json
selfiek validate-config --json
selfiek library lint --json
selfiek library report --json
selfiek library optimize --dry-run --json
selfiek feedback rate --image /path/to/used.png --score 2 --reason "natural" --like scene.concert,outfit.casual --dislike face_likeness --json
selfiek preference compile --json
selfiek preference report --json
selfiek preference evolve --dry-run --json
selfiek compile --json
selfiek compile --use-orderk --json
selfiek draw --json --use-templates --explain
selfiek generate --json --dry-run --use-templates  # manual stock generation only; writes to stock new_dir
selfiek produce --json --use-templates --dry-run
selfiek produce --json --use-templates --quiet
selfiek next --json --use-templates               # consumes stock only; does not generate on empty stock
selfiek cleanup-used --json
```

## Version model

- `runtime_version`: latest release
- `contract_version`: latest release

SelfieK 3.7 adds the Preference Engine: `feedback rate` records immutable feedback events linked to image sidecars and prompt cards; `preference compile` aggregates those events into `preference_model.json`, `preference_report.json`, and preference-aware `weights.json`; `preference report` explains positive/negative element trends; `preference evolve --dry-run` proposes candidate boosts/penalties without rewriting prompts. The hot path remains deterministic and offline: no LLM optimizer, no UI, no daemon, no MCP loop, and no cdper rewrite.

SelfieK 3.6 upgraded prompt-library absorption: `library lint` validates the Obsidian recipe book, `library ingest` creates source/template drafts, `library optimize --dry-run` produces an offline no-write cleanup plan, `library report` includes coverage and inventory-quality sections, `compile` emits `template_index.json`, `fragment_index.json`, `prompt_cards.jsonl`, `library_report.json`, and actionable `weights.json` v2, and `draw/generate --use-templates` attach `prompt_card` v2 for feedback attribution. orderk is compile/report-only; production cron still consumes compiled artifacts and cdper remains the image-generation adapter.

The prompt-library layer also includes lightweight guardrails borrowed from prompt-optimizer-style tooling without adopting its product shape: raw prompts are stored as JSON evidence references instead of copied wholesale, risky role/injection-like source text is reported by `library lint`, `{{placeholder}}` variables are tracked through prompt cards, structured JSON sources expose top-level key preservation warnings, feedback templates are nudged toward visible facts, and `library report` emits machine-readable `quality_signals`. There is still no LLM prompt optimizer, UI, daemon, MCP optimize tool, or cdper rewrite step.
