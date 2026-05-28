# SelfieK 3.5 Design

## Goal

SelfieK 3.5 turns the K/Kate selfie pipeline into a publishable Rust + TypeScript CLI while preserving the proven cdper image-generation adapter.

## Boundaries

- SelfieK owns deterministic orchestration: status, dice validation, template compilation, prompt-card draw, dry-run generation, stock consume, used cleanup, and cdper command assembly.
- cdper owns ChatGPT/CDP interaction and original image extraction. SelfieK must not reimplement browser automation.
- Obsidian owns source prompt/feedback knowledge.
- orderk remains a future read-only recall layer for compiler enrichment; it is not in the hot generation path.

## Runtime commands

```bash
selfiek status --json
selfiek validate-config --json
selfiek compile --json
selfiek draw --json --use-templates
selfiek generate --json --dry-run --use-templates
selfiek generate --json --use-templates
selfiek next --json
selfiek cleanup-used --json
```

## 3.5 activation level

SelfieK 3.5 activates the compiler path at CLI level:

1. `compile` reads Obsidian YAML, YML, and Markdown files with YAML frontmatter, then writes `template_index.json`.
2. `draw --use-templates` scores matching templates against the five dice and attaches a Prompt Card.
3. `generate --use-templates` injects selected transferable fragments into the final prompt and stores the Prompt Card in sidecar metadata.

Cron cutover should be conservative:

1. Install the npm CLI.
2. Run dry-run and one live smoke.
3. Update production wrappers to call `selfiek` only after smoke passes.
4. Keep old Python generator as rollback until several cron cycles are green.

## Release checklist

- Rust build/test pass.
- TS build pass.
- `selfiek compile --json` parses real Obsidian templates.
- `selfiek generate --dry-run --use-templates` creates reference collage and Prompt Card.
- `selfiek produce --use-templates --quiet` preserves stock limit, locks, batch pause, and silent healthy cron behavior.
- Real generation is serialized by `.selfiek.generation.lock` so direct smoke tests and cron do not fight over cdper/Chrome.
- npm tarball contains `dist/` and bundled `bin/selfiek-core-linux-x64`.
- GitHub release/tag verified.
- npm registry tarball and clean install verified.
