# selfiek

`selfiek` is the npm CLI for SelfieK 3.5.

```bash
npm install -g selfiek
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

The package bundles a Linux x64 Rust core binary. On other platforms set `SELFIEK_CORE_BIN` to a compatible `selfiek-core` binary.

SelfieK reuses `cdper-gpt-image` for actual ChatGPT/CDP image generation.
