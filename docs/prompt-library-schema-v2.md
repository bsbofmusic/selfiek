# SelfieK Prompt Library Schema v2

SelfieK 3.6 treats the Obsidian prompt library as the canonical recipe book and compiles it into lightweight runtime artifacts. Runtime generation and cron still consume compiled files only; they do not scan the whole vault and do not query orderk.

## Canonical layout

```text
raw/selfie-prompts/
  README.md
  inbox/
    external-prompts/
      watchlist/
      sources/
      candidates/
      rejected/
  sources/
  templates/
  fragments/
    scene/
    camera/
    composition/
    lighting/
    outfit/
    mood/
    effect/
    negative/
  feedback/
    positive/
    negative/
  rules/
    taxonomy.yaml
    compatibility.yaml
    safety.yaml
    style_weights.yaml
    k-image-tags.yaml
  reports/
```

K original/reference images must stay outside Obsidian, typically under `/home/agent/K-original/`. Obsidian may contain text metadata for those images, not image files.

## Source note

Raw prompts are immutable source notes:

```markdown
---
schema_version: selfiek.source.v1
id: src-old-digicam-escalator-20260528
status: active
type: raw_prompt_source
origin: user
created_at: "2026-05-28T10:00:00+08:00"
license_scope: personal_selfiek
---

# Old Digicam Escalator Prompt

## Raw Prompt

Original prompt text, preserved exactly.
```

## Template note

Curated recipes use Markdown plus YAML frontmatter:

```markdown
---
schema_version: selfiek.template.v2
id: tpl-old-digicam-escalator-20260528
title: 老式数码相机扶梯白月光抓拍
status: active
type: prompt_template
source:
  raw_prompt_path: raw/selfie-prompts/sources/src-old-digicam-escalator-20260528.md
  origin: user
  confidence: high
taxonomy:
  scene_ids: [scene.mall_escalator]
  style_ids: [style.old_digicam_ccd, style.candid_snapshot]
  camera_ids: [camera.high_angle_24mm]
  composition_ids: [composition.dutch_angle]
  outfit_ids: [outfit.white_sheer_cardigan]
  mood_ids: [mood.white_moonlight, mood.nostalgic_urban]
compiler:
  use_mode: fragments
  priority: high
  max_fragments_per_card: 4
  avoid_full_prompt_copy: true
compatibility:
  preferred_scene_ids: [scene.mall_escalator, scene.mall]
  forbidden_scene_ids: [scene.office, scene.bathroom]
safety:
  boundary: daily_fashion
  avoid_oversexualization: true
---

# 老式数码相机扶梯白月光抓拍

## Summary
A concise human summary.

## Must Keep
- Reusable fragment that should survive recomposition.

## Optional
- Optional camera/lighting/mood/outfit detail.

## Avoid
- Failure modes or overfitting risks.
```

## Harvest candidate note

`selfiek library harvest --source <path> --dry-run|--apply` ingests local watchlist/search-export notes into `inbox/external-prompts/`. Accepted items get immutable source notes under `inbox/external-prompts/sources/` plus candidate notes under `inbox/external-prompts/candidates/`; low-scoring/high-risk items go to `inbox/external-prompts/rejected/` for dedupe evidence. These notes are **not** scanned as active templates and never reach runtime generation until a human promotes them into `templates/` and reruns lint/compile.

```markdown
---
schema_version: selfiek.harvest_candidate.v1
id: cand-fisheye-vaporwave-subway-1234abcd
status: triaged
type: prompt_template_candidate
source:
  source_url: https://example.invalid/reference
  source_type: watchlist_seed
candidate_status: triaged
quality_dimensions: [camera_authenticity, background_realism, skin_texture]
style_tags: [fisheye, vaporwave, candid_snapshot]
fun_axes: [novelty, persona_fit, remixability]
quality_score: 4.2
fun_score: 4.1
risk_score: 0.8
policy: candidate_only_no_generation_no_runtime_weight_write
---
```

## Fragment note

Reusable atoms can live as YAML or Markdown frontmatter:

```yaml
schema_version: selfiek.fragment.v1
id: effect.vintage_orange_date_stamp
category: effect
status: active
text_zh: 左下角复古橙色日期戳，像 2000 年代消费级数码相机直出
text_en: a small vintage orange date stamp in the lower-left corner
tags: [old_digicam, nostalgia, candid]
compatible_style_ids: [style.old_digicam_ccd, style.candid_snapshot]
avoid_with: [style.clean_digital, style.fashion_editorial]
```

## CLI contract

```bash
selfiek library lint --json
selfiek library ingest --source <path-or-dir> --dry-run --json
selfiek library ingest --source <path-or-dir> --apply --json
selfiek library harvest --source <watchlist-or-search-export> --dry-run --json
selfiek library harvest --source <watchlist-or-search-export> --apply --json
selfiek library report --json
selfiek compile --use-orderk --json
selfiek draw --use-templates --explain --json
selfiek generate --dry-run --use-templates --json
```

## Runtime artifacts

`selfiek compile` writes:

```text
template_index.json
fragment_index.json
prompt_cards.jsonl
library_report.json
weights.json
```

`draw` and `generate` attach `prompt_card.schema_version = selfiek.prompt_card.v2` when `--use-templates` finds a matching compiled template.

## Lint semantics

- v2 templates are strictly checked for schema, source link, taxonomy IDs, and `compiler.use_mode`.
- Legacy YAML templates are accepted but reported as `legacy_needs_migration`.
- Adult/explicit boundary words in old library metadata are reported as `boundary_noise_in_library_text`; generation prompts continue to use SelfieK's clean anatomy/quality negative suffix.
- Original K image files inside the prompt library are hard errors.

## orderk boundary

orderk is a read-only semantic librarian. SelfieK 3.6 may probe it during `compile --use-orderk` / report generation and records availability, but runtime `draw`, `generate`, `produce`, `next`, and cron do not call orderk.
