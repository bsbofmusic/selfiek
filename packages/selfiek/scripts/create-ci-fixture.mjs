#!/usr/bin/env node
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const root = process.argv[2] || process.env.SELFIEK_CI_ROOT || '/tmp/selfiek-ci-fixture';
const dirs = {
  root,
  promptLib: join(root, 'prompt-lib'),
  runtime: join(root, 'runtime'),
  kOriginal: join(root, 'K-original'),
  newDir: join(root, 'new'),
  usedDir: join(root, 'used'),
};
for (const dir of Object.values(dirs)) mkdirSync(dir, { recursive: true });
for (const rel of [
  'prompt-lib/sources',
  'prompt-lib/templates',
  'prompt-lib/fragments/effect',
  'prompt-lib/rules',
]) mkdirSync(join(root, rel), { recursive: true });

const tinyJpeg = Buffer.from(
  '/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAgGBgcGBQgHBwcJCQgKDBQNDAsLDBkSEw8UHRofHh0aHBwgJC4nICIsIxwcKDcpLDAxNDQ0Hyc5PTgyPC4zNDL/2wBDAQkJCQwLDBgNDRgyIRwhMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjL/wAARCAAIAAgDASIAAhEBAxEB/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/8QAHwEAAwEBAQEBAQEBAQAAAAAAAAECAwQFBgcICQoL/8QAtREAAgECBAQDBAcFBAQAAQJ3AAECAxEEBSExBhJBUQdhcRMiMoEIFEKRobHBCSMzUvAVYnLRChYkNOEl8RcYGRomJygpKjU2Nzg5OkNERUZHSElKU1RVVldYWVpjZGVmZ2hpanN0dXZ3eHl6goOEhYaHiImKkpOUlZaXmJmaoqOkpaanqKmqsrO0tba3uLm6wsPExcbHyMnK0tPU1dbX2Nna4uPk5ebn6Onq8vP09fb3+Pn6/9oADAMBAAIRAxEAPwDvqKKK4D1D/9k=',
  'base64',
);
for (let i = 1; i <= 3; i++) writeFileSync(join(dirs.kOriginal, `k-${i}.jpg`), tinyJpeg);

writeFileSync(
  join(root, 'dice_config.json'),
  JSON.stringify(
    {
      version: 'ci-fixture-3.6.0',
      scenes: [
        {
          id: 1,
          name: '🎤 演唱会现场',
          prompt: '演唱会舞台灯和荧光棒背景',
          openings: ['茶老板，今晚灯光有点好看。'],
        },
      ],
      styles: [{ id: 1, name: '被偷拍/被抓拍', prompt: 'candid snapshot, phone photo' }],
      outfits: [{ id: 1, name: '日常休闲', prompt: 'casual daily outfit' }],
      compatible_style_ids: { '1': [1] },
      compatible_outfit_ids: { '1': [1] },
      film_styles: ['Kodak Gold 200 film grain'],
      lighting_styles: ['soft stage light'],
    },
    null,
    2,
  ),
);

writeFileSync(
  join(dirs.promptLib, 'sources/src-demo.md'),
  `---\nschema_version: selfiek.source.v1\nid: src-demo\nstatus: active\ntype: raw_prompt_source\n---\n\n## Raw Prompt\n演唱会舞台灯抓拍。\n`,
);
writeFileSync(
  join(dirs.promptLib, 'rules/taxonomy.yaml'),
  `schema_version: selfiek.taxonomy.v1\nscenes:\n  - id: scene.concert\nstyles:\n  - id: style.candid_snapshot\ncameras:\n  - id: camera.phone\ncompositions:\n  - id: composition.closeup\noutfits:\n  - id: outfit.casual\nmoods:\n  - id: mood.energetic\neffects:\n  - id: effect.stage_light\n`,
);
writeFileSync(join(dirs.promptLib, 'rules/safety.yaml'), `schema_version: selfiek.safety.v1\n`);
writeFileSync(
  join(dirs.promptLib, 'templates/tpl-demo.md'),
  `---\nschema_version: selfiek.template.v2\nid: tpl-demo\ntitle: 演唱会抓拍\nstatus: active\ntype: prompt_template\nsource:\n  raw_prompt_path: sources/src-demo.md\ntaxonomy:\n  scene_ids: [scene.concert]\n  style_ids: [style.candid_snapshot]\n  camera_ids: [camera.phone]\n  composition_ids: [composition.closeup]\n  outfit_ids: [outfit.casual]\n  mood_ids: [mood.energetic]\n  effect_ids: [effect.stage_light]\ncompiler:\n  use_mode: fragments\n  priority: high\n---\n\n# 演唱会抓拍\n\n## Must Keep\n- 彩色舞台灯扫过脸侧\n- 人群背景里有荧光棒\n\n## Avoid\n- 专业棚拍感\n`,
);

console.log(JSON.stringify(dirs, null, 2));
