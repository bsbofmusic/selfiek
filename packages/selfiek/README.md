# selfiek

`selfiek` is the npm CLI for SelfieK 3.6.

```bash
npm install -g selfiek
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

The package bundles a Linux x64 Rust core binary. On other platforms set `SELFIEK_CORE_BIN` to a compatible `selfiek-core` binary.

SelfieK reuses `cdper-gpt-image` for actual ChatGPT/CDP image generation.

Prompt-library guardrails are intentionally lightweight: `library lint/report` can surface prompt-injection-like raw source text, raw-copy risk, placeholder preservation state, structured JSON key preservation hints, and feedback visible-fact warnings. The package does not include an LLM prompt optimizer, UI, daemon, MCP optimize tool, or image-generation backend.
