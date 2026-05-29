# selfiek

`selfiek` is the npm CLI for SelfieK.

```bash
npm install -g selfiek
selfiek status --json
selfiek profile show --json
selfiek profile validate --file ./profile.yml --json
selfiek validate-config --json
selfiek library lint --json
selfiek library report --json
selfiek library optimize --dry-run --json
selfiek library harvest --source /path/to/watchlist-or-search-export --dry-run --json
selfiek library harvest --source /path/to/watchlist-or-search-export --apply --json
selfiek feedback rate --image /path/to/used.png --score 2 --reason "natural" --like scene.concert,outfit.casual --dislike face_likeness --json
selfiek preference compile --json
selfiek preference report --json
selfiek preference evolve --dry-run --json
selfiek compile --json
selfiek compile --use-orderk --json
selfiek draw --json --use-templates --explain
selfiek generate --json --dry-run --use-templates  # manual stock generation only
selfiek produce --json --use-templates --dry-run
selfiek produce --json --use-templates --quiet
selfiek next --json --use-templates               # consumes stock only
selfiek cleanup-used --json
```

The package bundles a Linux x64 Rust core binary. On other platforms set `SELFIEK_CORE_BIN` to a compatible `selfiek-core` binary.

SelfieK reuses `cdper-gpt-image` for actual ChatGPT/CDP image generation during stock production. It intentionally does not provide an instant/ad-hoc image lane; one-off image requests should use cdper directly.

Stock-only profiles can be supplied with `--profile-file` / `SELFIEK_PROFILE_FILE`:

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

Prompt-library guardrails are intentionally lightweight: `library lint/report` can surface prompt-injection-like raw source text, raw-copy risk, placeholder preservation state, structured JSON key preservation hints, feedback visible-fact warnings, coverage gaps, and inventory sidecar quality. `library harvest --source <path> --dry-run|--apply` is an offline intake lane for external prompt references: it scores quality/fun/risk and writes review-only notes under `inbox/external-prompts`, without generating images, rewriting prompts, updating weights, or calling cdper. `library optimize --dry-run` returns an offline no-write plan; it does not rewrite prompts automatically. Preference Engine commands (`feedback rate`, `preference compile/report/evolve --dry-run`) learn from explicit photo feedback by compiling immutable feedback events into offline weights and reports. The package does not include an LLM prompt optimizer, UI, daemon, MCP optimize tool, or image-generation backend.
