# selfiek

`selfiek` is the npm CLI for SelfieK.

```bash
npm install -g selfiek
selfiek status --json
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
selfiek generate --json --dry-run --use-templates
selfiek produce --json --use-templates --dry-run
selfiek produce --json --use-templates --quiet
selfiek next --json --use-templates
selfiek cleanup-used --json
```

The package bundles a Linux x64 Rust core binary. On other platforms set `SELFIEK_CORE_BIN` to a compatible `selfiek-core` binary.

SelfieK reuses `cdper-gpt-image` for actual ChatGPT/CDP image generation.

Prompt-library guardrails are intentionally lightweight: `library lint/report` can surface prompt-injection-like raw source text, raw-copy risk, placeholder preservation state, structured JSON key preservation hints, feedback visible-fact warnings, coverage gaps, and inventory sidecar quality. `library optimize --dry-run` returns an offline no-write plan; it does not rewrite prompts automatically. Preference Engine commands (`feedback rate`, `preference compile/report/evolve --dry-run`) learn from explicit photo feedback by compiling immutable feedback events into offline weights and reports. The package does not include an LLM prompt optimizer, UI, daemon, MCP optimize tool, or image-generation backend.
