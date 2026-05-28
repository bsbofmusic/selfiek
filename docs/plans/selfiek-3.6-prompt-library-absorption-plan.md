# SelfieK 3.6 Prompt Library Absorption Plan

> **For Hermes:** this is a design/implementation plan. Do not implement until the plan has passed GPT audit and 茶老板 approves execution.

**Goal:** Fix the 3.5 gap: make SelfieK truly absorb 茶老板's high-quality prompt library and K image library as structured, reusable, searchable, composable assets.

**Architecture:** Obsidian remains the canonical knowledge source, orderk is a read-only semantic retrieval knife used at compile/audit time, SelfieK compiler emits lightweight runtime artifacts, and cdper continues to own ChatGPT/CDP image generation. Runtime cron must stay fast and stable: it consumes compiled artifacts only, not the full Obsidian vault or live orderk queries.

**Tech Stack:** Rust core + TypeScript npm CLI (`selfiek`), Obsidian Markdown/YAML files, orderk CLI/search DB, existing `cdper-gpt-image`.

---

## 0. Context Recovered

茶老板's original requirement was not merely "read a few YAML templates." It was:

1. There are two manually supplied assets:
   - K image library.
   - High-quality prompt library.
2. The prompt library is like recipes/raw material. SelfieK must classify, tag, split, and recombine it into new usable selfies.
3. Obsidian and orderk must have clear roles.
4. The latest version must actually be deployed, automated, not conflict with other tools, and continue using cdper.
5. The whole scheme is named SelfieK / `selfiek`.

SelfieK 3.5 delivered Rust+TS CLI, npm/GitHub release, stock/cron locks, basic compiler v0, and cdper reuse. But it under-delivered on the core prompt-library absorption: current compile only reads the local Obsidian `raw/selfie-prompts/templates/` templates and does shallow fragment extraction. It does not yet provide a rigorous ingestion, normalization, controlled taxonomy, orderk-assisted dedup/gap analysis, K-image metadata matching, or feedback attribution loop.

## 1. Current Problem

Current facts:

- `selfiek status --json` reports `paths.prompt_lib = /home/agent/obsidian-vault/raw/selfie-prompts`.
- Current template index contains about 10 local templates.
- A typical template such as `templates/old-digital-camera-escalator-girl-20260528.yaml` is rich but too fat: raw prompt, tags, fragments, full prompt, negative, notes, safety, and randomizable fields live in one 8KB YAML.
- This is usable as a seed, but not a scalable prompt library.

Main failure mode: SelfieK 3.5 can say "a template was used," but cannot honestly say "茶老板's prompt library has been absorbed as a durable, structured, self-improving recipe system."

## 2. Design Principles

### 2.1 Roles

- **Obsidian = recipe book / source of truth**
  - Stores raw prompts, curated templates, fragments, rules, positive/negative feedback, and human-readable design notes.
  - Markdown is preferred for human-readable knowledge; YAML frontmatter is used for machine fields.

- **orderk = read-only librarian**
  - Finds similar templates/fragments.
  - Detects duplicates.
  - Finds missing coverage/gaps.
  - Helps attribution after positive/negative feedback.
  - Never writes files or changes weights by itself.

- **SelfieK compiler = prep cook**
  - Reads Obsidian prompt library.
  - Uses deterministic schema/lint rules.
  - Optionally consults orderk during compile/audit, not runtime.
  - Produces lightweight runtime artifacts.

- **SelfieK runtime = serving machine**
  - Draws dice, selects precompiled prompt cards/fragments, calls cdper, writes sidecar JSON.
  - Must not scan the full vault or query orderk in cron hot path.

- **cdper = photographer**
  - Continues to own ChatGPT/CDP browser image generation.
  - SelfieK must not reimplement browser automation.

### 2.2 Non-goals

- Do not rewrite cdper.
- Do not move K original images into Obsidian.
- Do not let orderk generate prompts, mutate weights, or run in every production cron tick.
- Do not let LLM rewrite the prompt library automatically during runtime.
- Do not break the current 3.5 stock/cron path while improving the library.

## 3. Proposed Obsidian Library Structure

Target path stays under:

```text
/home/agent/obsidian-vault/raw/selfie-prompts/
```

New normalized layout:

```text
raw/selfie-prompts/
  README.md

  inbox/
    # newly supplied prompts before normalization

  sources/
    # immutable original prompts, one prompt per .md

  templates/
    # curated recipe-level templates, .md + YAML frontmatter preferred

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
    compiler-report-YYYYMMDD.md
```

## 4. Schema v2

### 4.1 Source note

One raw prompt becomes an immutable source note:

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

完整原始提示词，不改写，不删减。
```

### 4.2 Template note

Curated templates become compact Markdown notes with machine-readable frontmatter:

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
商场扶梯，高处俯拍，CCD 数码噪点，荷兰角，橙色日期戳，冷白荧光灯，怀旧都市抓拍。

## Must Keep
- 老式数码相机 CCD 数字噪点
- 高处俯拍约 24mm
- 公共扶梯下行
- 荷兰角不完美构图
- 左下角橙色复古日期戳

## Optional
- 背景模糊行人
- 冷白荧光灯
- 白色轻薄外套
- 黑色皮质手提包

## Avoid
- 专业棚拍感
- 过度精修皮肤
- 时尚大片感
- 过度暴露或擦边
```

### 4.3 Fragment note

Reusable atoms get stable IDs:

```yaml
schema_version: selfiek.fragment.v1
id: effect.vintage_orange_date_stamp
category: effect
status: active
text_zh: 左下角复古橙色日期戳，像 2000 年代消费级数码相机直出
text_en: a small vintage orange date stamp in the lower-left corner, like a 2000s consumer digital camera snapshot
tags: [old_digicam, nostalgia, candid]
compatible_style_ids: [style.old_digicam_ccd, style.candid_snapshot]
avoid_with: [style.clean_digital, style.fashion_editorial]
```

### 4.4 K image metadata

Images remain in `/home/agent/K-original/`; only metadata lives in Obsidian/rules:

```yaml
schema_version: selfiek.k_image_tags.v1
images:
  - path: /home/agent/K-original/k-001.jpg
    tags: [front_face, soft_smile, long_hair, clean_reference]
    best_for: [mood.white_moonlight, style.soft_lifestyle]
    avoid_for: [camera.extreme_profile]
```

This turns the K image library from purely random 3-image collage into tag-aware reference selection without storing images in Obsidian.

## 5. Compiler 3.6 Responsibilities

Add/upgrade commands:

```bash
selfiek library lint --json
selfiek library ingest --source <path-or-dir> --dry-run --json
selfiek library ingest --source <path-or-dir> --apply --json
selfiek library report --json
selfiek compile --use-orderk --json
selfiek draw --use-templates --explain --json
```

### 5.1 Ingest

Input can be a directory, a Markdown file, YAML files, or exported prompt pack.

Pipeline:

```text
raw prompt(s)
  -> source note(s)
  -> draft template(s)
  -> fragment candidates
  -> taxonomy mapping
  -> lint report
  -> human-readable report
```

Ingest must be deterministic and conservative. It can generate draft structure, but it must preserve raw prompts exactly. If LLM extraction is ever used, it is only for draft classification, not source rewriting.

### 5.2 Lint

Library lint checks:

- Every source/template/fragment has `schema_version` and stable `id`.
- Every template points to an existing raw source.
- Every taxonomy ID exists in `rules/taxonomy.yaml`.
- Every template has `compiler.use_mode`.
- No template stores huge full prompt blobs unless in `sources/`.
- Fragment IDs are unique.
- Safety/negative rules exist.
- No image paths point into Obsidian for original K images.
- Legacy YAML v1 templates are accepted but reported as `legacy_needs_migration`.

### 5.3 orderk-assisted compile

orderk is used only in compile/report stage:

- Similarity dedup: "this new prompt is 92% like existing old-digicam-escalator".
- Candidate retrieval: for a dice draw, retrieve semantically aligned templates/fragments.
- Gap report: under-covered scene/style/outfit areas.
- Feedback attribution: find templates/fragments similar to a praised or rejected sidecar.

If orderk is unavailable, compile still works via deterministic taxonomy matching, but report says `orderk_available=false`.

### 5.4 Runtime artifacts

Compiler writes:

```text
/home/agent/.hermes/scripts/k-selfie-generator/template_index.json
/home/agent/.hermes/scripts/k-selfie-generator/fragment_index.json
/home/agent/.hermes/scripts/k-selfie-generator/prompt_cards.jsonl
/home/agent/.hermes/scripts/k-selfie-generator/library_report.json
/home/agent/.hermes/scripts/k-selfie-generator/weights.json
```

Runtime reads these only. Cron remains stable.

## 6. Prompt Card Composition

When `selfiek draw --use-templates` runs:

1. Draw dice from `dice_config.json`.
2. Map scene/style/outfit to taxonomy IDs.
3. Select candidate templates by:
   - hard compatibility;
   - taxonomy score;
   - orderk compile-time similarity score if available;
   - positive/negative weights;
   - novelty penalty to avoid repeating the same template too often.
4. Select 2-5 fragments, not whole raw prompts by default.
5. Choose K reference images using metadata tags when available; fallback to random 3 images.
6. Emit a `prompt_card` with all template/fragment/source IDs.

Sidecar JSON must include:

```json
{
  "prompt_card": {
    "schema_version": "selfiek.prompt_card.v2",
    "template_ids": [],
    "fragment_ids": [],
    "source_ids": [],
    "k_image_ids": [],
    "taxonomy_ids": [],
    "weights_applied": [],
    "negative_rules": []
  }
}
```

This is critical: without prompt_card in sidecar, positive/negative feedback cannot be attributed.

## 7. Feedback Loop

Positive feedback:

```text
user praises image
  -> locate sidecar
  -> vision_analyze actual image
  -> create feedback/positive/*.md
  -> link prompt_card template/fragment IDs
  -> next compile updates weights.json
```

Negative feedback:

```text
user rejects image
  -> locate sidecar
  -> vision_analyze actual image
  -> create feedback/negative/*.md
  -> classify root cause: face / scene / outfit / composition / lighting / anatomy / text mismatch / model drift
  -> next compile downgrades or forbids matching fragments
```

No feedback should be written without seeing the actual image when the feedback refers to a specific picture.

## 8. Migration Strategy

P0 safe migration, no cron breakage:

1. Snapshot current `raw/selfie-prompts/` and runtime artifacts.
2. Add schema v2 support while keeping legacy YAML support.
3. Create `rules/taxonomy.yaml` and map existing dice scenes/styles/outfits to canonical IDs.
4. Convert 1 representative template (`old-digital-camera-escalator-girl`) into source + template v2 + fragments.
5. Add linter and fixture tests.
6. Compile both legacy and v2 templates.
7. Verify `draw --use-templates --explain` shows v2 template/fragment IDs.
8. Keep current production cron unchanged until compile/draw tests pass.

P1 useful library absorption:

1. Add `selfiek library ingest` for external prompt packs.
2. Ingest 茶老板's canonical prompt library into `sources/` and draft `templates/`.
3. Run lint and compile.
4. Add sidecar prompt_card v2.
5. Add feedback positive/negative notes linked to prompt_card IDs.

P2 quality growth:

1. Add K image metadata tagging.
2. Add orderk gap/dedup report.
3. Add quality dashboard: templates used, praised, rejected, stale, overused.
4. Add prompt-card novelty balancing.

## 9. Acceptance Criteria

SelfieK can only claim "茶老板's prompt library is used" when all of these are true:

1. `selfiek status --json` reports the expected canonical `paths.prompt_lib`.
2. `selfiek library lint --json` passes or reports only known legacy warnings.
3. `template_index.json` includes paths from the canonical prompt library.
4. `fragment_index.json` includes reusable fragments extracted from that library.
5. `selfiek draw --use-templates --explain --json` returns prompt_card with template/fragment/source IDs from the imported library.
6. A dry-run `selfiek generate --dry-run --use-templates --json` shows the selected fragments in the generated prompt.
7. New sidecar JSON from real or dry-run generation includes `prompt_card` v2.
8. orderk integration is compile/report-only and disabled or skipped cleanly if unavailable.
9. Existing cron wrappers still succeed silently on healthy produce/cleanup.
10. GPT + MiMo audits pass before publishing 3.6.

## 10. Score Impact

Current revised SelfieK 3.5 score after discovering the prompt-library gap: about 8.5/10.

Expected after P0+P1:

- Prompt-library absorption: 6.5 -> 8.8
- Feedback attribution: 6.0 -> 8.4
- Maintainability: 8.8 -> 9.0
- Runtime stability: should remain >=8.7 because runtime stays precompiled and cdper unchanged.

Projected total: 9.0-9.2/10 if implemented with tests and audits.

## 11. Implementation Tasks

### Task 1: Add schema docs and fixtures

Files:
- Create `docs/prompt-library-schema-v2.md`
- Create `tests/fixtures/prompt-library-v2/`

Verify:
- Fixtures include source/template/fragment/feedback/rules examples.

### Task 2: Add library linter

Files:
- Modify `crates/selfiek-core/src/main.rs` or split into modules if it grows.
- Add command `library lint`.

Tests:
- Missing schema fails.
- Missing raw source path fails.
- Unknown taxonomy ID fails.
- Legacy v1 YAML emits warning but does not fail.

### Task 3: Add v2 parser

Support:
- `.md` with YAML frontmatter + sections.
- Pure `.yaml` fragments/rules.
- Legacy templates.

Tests:
- Parse source note.
- Parse template note.
- Parse fragment YAML.
- Extract Must Keep / Optional / Avoid sections.

### Task 4: Add controlled taxonomy

Files:
- Create `raw/selfie-prompts/rules/taxonomy.yaml`.
- Map current dice scenes/styles/outfits to IDs.

Tests:
- `selfiek validate-config` checks dice IDs can map to taxonomy.

### Task 5: Add compile artifacts

Output:
- `fragment_index.json`
- `prompt_cards.jsonl`
- `library_report.json`
- `weights.json`

Tests:
- Atomic write.
- Correct template/fragment counts.
- No full raw prompt in runtime artifacts unless `use_mode=full_template`.

### Task 6: Add prompt_card v2 in draw/generate sidecar

Tests:
- `draw --use-templates --explain --json` includes prompt_card v2.
- `generate --dry-run --use-templates --json` includes prompt_card v2.
- Existing fields remain backward compatible.

### Task 7: Add orderk compile/report integration

Rules:
- No runtime orderk calls in `produce` hot path.
- If orderk unavailable, report warning only.

Tests:
- Mock orderk output.
- `compile --use-orderk --json` includes dedup/gap candidates.

### Task 8: Add ingest command

Command:

```bash
selfiek library ingest --source <path> --dry-run --json
selfiek library ingest --source <path> --apply --json
```

Tests:
- Import directory of raw prompts.
- Preserve original prompt exactly.
- Generate draft v2 templates with stable IDs.
- No write on dry-run.

### Task 9: Migrate representative template

Migrate:
- `old-digital-camera-escalator-girl-20260528.yaml`

Into:
- `sources/src-old-digicam-escalator-20260528.md`
- `templates/tpl-old-digicam-escalator-20260528.md`
- several `fragments/.../*.yaml`

Verify:
- legacy and v2 coexist.
- compile picks v2 preferentially.

### Task 10: Full verification and release gate

Run:

```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
npm test --workspace selfiek
selfiek library lint --json
selfiek compile --use-orderk --json
selfiek draw --use-templates --explain --json
selfiek generate --dry-run --use-templates --json
python3 /home/agent/.hermes/scripts/k-selfie-produce.py
```

Then run GPT + MiMo audits before any GitHub/npm release.
