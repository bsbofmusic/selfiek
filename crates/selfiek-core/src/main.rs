use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Duration, Local, Utc};
use clap::{Parser, Subcommand};
use fs2::FileExt;
use image::{DynamicImage, GenericImage, ImageBuffer, Rgb, RgbImage};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

const VERSION: &str = "3.5.0";
const REF_IMAGE_PREFIX: &str = "请直接生成一张全新的写实摄影风格图片，不要回复文字，不要解释。参考拼图包含 3 张同一角色照片，仅作为面部特征和发型气质参考。请完全忽略参考图中的服装、姿势、背景、光线和拍摄角度。你需要生成一个全新的、独立的场景和构图，只保留图中人物的稳定面部特征（脸型、五官比例、肤质、发色）。";
const NEGATIVE_SUFFIX: &str = "\n\n[Important constraints] Must avoid: deformed fingers, extra fingers, fused fingers, backwards fingers, mutated hands, poorly drawn hands, malformed limbs, extra arms, extra legs, fused legs, too many fingers, long neck, distorted face, asymmetric eyes, cross-eyed, cloned face, ugly, disfigured, blurry, low quality, pixelated, watermark, text overlay, over-processed, plastic skin, waxy appearance, uncanny valley, AI artifacts, unrealistic proportions, cartoon, anime, 3d render.";

#[derive(Parser, Debug)]
#[command(name = "selfiek-core", version = VERSION, about = "Rust core for SelfieK")]
struct Cli {
    #[arg(
        long,
        global = true,
        env = "SELFIEK_DICE_CONFIG",
        default_value = "/home/agent/.hermes/scripts/k-selfie-generator/dice_config.json"
    )]
    dice_config: PathBuf,
    #[arg(
        long,
        global = true,
        env = "SELFIEK_K_ORIGINAL",
        default_value = "/home/agent/K-original"
    )]
    k_original: PathBuf,
    #[arg(
        long,
        global = true,
        env = "SELFIEK_NEW_DIR",
        default_value = "/home/agent/k-selfie-new"
    )]
    new_dir: PathBuf,
    #[arg(
        long,
        global = true,
        env = "SELFIEK_USED_DIR",
        default_value = "/home/agent/k-selfie-used"
    )]
    used_dir: PathBuf,
    #[arg(
        long,
        global = true,
        env = "SELFIEK_PROMPT_LIB",
        default_value = "/home/agent/obsidian-vault/raw/selfie-prompts"
    )]
    prompt_lib: PathBuf,
    #[arg(
        long,
        global = true,
        env = "SELFIEK_RUNTIME_DIR",
        default_value = "/home/agent/.hermes/scripts/k-selfie-generator"
    )]
    runtime_dir: PathBuf,
    #[arg(
        long,
        global = true,
        env = "SELFIEK_CDPER_BIN",
        default_value = "/home/agent/.local/bin/cdper-gpt-image"
    )]
    cdper_bin: PathBuf,
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Status,
    ValidateConfig,
    Compile {
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Draw {
        #[arg(long)]
        scene: Option<u32>,
        #[arg(long)]
        style: Option<u32>,
        #[arg(long)]
        outfit: Option<u32>,
        #[arg(long)]
        use_templates: bool,
    },
    Generate {
        #[arg(long)]
        scene: Option<u32>,
        #[arg(long)]
        style: Option<u32>,
        #[arg(long)]
        outfit: Option<u32>,
        #[arg(long)]
        use_templates: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        quiet: bool,
        #[arg(long)]
        out_dir: Option<PathBuf>,
    },
    Produce {
        #[arg(long)]
        use_templates: bool,
        #[arg(long)]
        quiet: bool,
        #[arg(long)]
        dry_run: bool,
    },
    Next {
        #[arg(long)]
        use_templates: bool,
    },
    CleanupUsed {
        #[arg(long, default_value_t = 7)]
        days: i64,
    },
    Version,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct DiceConfig {
    version: Option<String>,
    scenes: Vec<Scene>,
    styles: Vec<Style>,
    outfits: Vec<Outfit>,
    compatible_style_ids: Option<HashMap<String, Vec<u32>>>,
    compatible_outfit_ids: Option<HashMap<String, Vec<u32>>>,
    film_styles: Vec<String>,
    lighting_styles: Vec<String>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Scene {
    id: u32,
    name: String,
    prompt: String,
    openings: Vec<String>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Style {
    id: u32,
    name: String,
    prompt: String,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Outfit {
    id: u32,
    name: String,
    prompt: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct TemplateEntry {
    path: String,
    id: String,
    name: Option<String>,
    template_type: Option<String>,
    scene_tags: Vec<String>,
    style_tags: Vec<String>,
    outfit_tags: Vec<String>,
    mood_tags: Vec<String>,
    fragment_texts: Vec<String>,
    avoid: Vec<String>,
    positive_weight: Option<String>,
    negative_weight: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct TemplateIndex {
    schema: String,
    version: String,
    generated_at: String,
    source_dir: String,
    template_count: usize,
    templates: Vec<TemplateEntry>,
}

#[derive(Debug, Serialize)]
struct DrawResult {
    ok: bool,
    drawn_at: String,
    scene: IdName,
    style: IdName,
    outfit: IdName,
    k_images: Vec<String>,
    opening: String,
    film_style: String,
    lighting: String,
    scene_prompt: String,
    style_prompt: String,
    outfit_prompt: String,
    compatibility_warning: Option<String>,
    full_prompt: String,
    prompt_card: Option<Value>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct IdName {
    id: u32,
    name: String,
}

fn main() {
    if let Err(e) = run() {
        println!("{}", json!({"ok": false, "error": e.to_string()}));
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Status => emit(status(&cli)?, true),
        Commands::ValidateConfig => emit(validate_report(&cli.dice_config)?, true),
        Commands::Compile { out } => emit(compile_templates(&cli, out.clone())?, true),
        Commands::Draw {
            scene,
            style,
            outfit,
            use_templates,
        } => emit(
            json!(draw(&cli, *scene, *style, *outfit, *use_templates)?),
            true,
        ),
        Commands::Generate {
            scene,
            style,
            outfit,
            use_templates,
            dry_run,
            quiet,
            out_dir,
        } => emit(
            generate(
                &cli,
                *scene,
                *style,
                *outfit,
                *use_templates,
                *dry_run,
                *quiet,
                out_dir.clone(),
            )?,
            true,
        ),
        Commands::Produce {
            use_templates,
            quiet,
            dry_run,
        } => emit(produce(&cli, *use_templates, *quiet, *dry_run)?, true),
        Commands::Next { use_templates } => emit(next(&cli, *use_templates)?, true),
        Commands::CleanupUsed { days } => emit(cleanup_used(&cli, *days)?, true),
        Commands::Version => emit(
            json!({"ok": true, "name": "selfiek", "version": VERSION}),
            true,
        ),
    }
    Ok(())
}

fn emit(v: Value, _json: bool) {
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
}

fn ensure_dirs(cli: &Cli) -> Result<()> {
    fs::create_dir_all(&cli.k_original)?;
    fs::create_dir_all(&cli.new_dir)?;
    fs::create_dir_all(&cli.used_dir)?;
    fs::create_dir_all(&cli.runtime_dir)?;
    Ok(())
}
fn image_files(dir: &Path) -> Vec<PathBuf> {
    let mut xs: Vec<_> = fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file())
        .filter(|p| {
            matches!(
                p.extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase()
                    .as_str(),
                "png" | "jpg" | "jpeg" | "webp"
            )
        })
        .collect();
    xs.sort_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).ok());
    xs
}
fn read_json_file<T: for<'de> Deserialize<'de>>(p: &Path) -> Result<T> {
    let s = fs::read_to_string(p).with_context(|| format!("read {}", p.display()))?;
    serde_json::from_str(&s).with_context(|| format!("parse json {}", p.display()))
}
fn dice(cli: &Cli) -> Result<DiceConfig> {
    read_json_file(&cli.dice_config)
}

fn validate_report(path: &Path) -> Result<Value> {
    if !path.exists() {
        return Ok(
            json!({"ok": false, "errors": [format!("missing dice config: {}", path.display())], "config": path}),
        );
    }
    let cfg: DiceConfig = read_json_file(path)?;
    let mut errors = vec![];
    if cfg.scenes.is_empty() {
        errors.push("scenes is empty".to_string());
    }
    if cfg.styles.is_empty() {
        errors.push("styles is empty".to_string());
    }
    if cfg.outfits.is_empty() {
        errors.push("outfits is empty".to_string());
    }
    if cfg.film_styles.is_empty() {
        errors.push("film_styles is empty".to_string());
    }
    if cfg.lighting_styles.is_empty() {
        errors.push("lighting_styles is empty".to_string());
    }
    let mut scene_ids = std::collections::HashSet::new();
    for s in &cfg.scenes {
        if !scene_ids.insert(s.id) {
            errors.push(format!("duplicate scene id {}", s.id));
        }
        if s.name
            .chars()
            .next()
            .map(|c| c.is_alphanumeric())
            .unwrap_or(true)
        {
            errors.push(format!(
                "scene {} name should start with emoji/symbol",
                s.id
            ));
        }
        if s.openings.is_empty() {
            errors.push(format!("scene {} has no openings", s.id));
        }
    }
    let style_ids: std::collections::HashSet<_> = cfg.styles.iter().map(|s| s.id).collect();
    let outfit_ids: std::collections::HashSet<_> = cfg.outfits.iter().map(|o| o.id).collect();
    for (sid, ids) in cfg.compatible_style_ids.clone().unwrap_or_default() {
        if sid.parse::<u32>().is_err() {
            errors.push(format!("invalid style map scene key {sid}"));
        }
        for id in ids {
            if !style_ids.contains(&id) {
                errors.push(format!(
                    "compatible_style_ids scene {sid} unknown style {id}"
                ));
            }
        }
    }
    for s in &cfg.scenes {
        if !cfg
            .compatible_outfit_ids
            .clone()
            .unwrap_or_default()
            .contains_key(&s.id.to_string())
        {
            errors.push(format!("missing compatible_outfit_ids for scene {}", s.id));
        }
    }
    for (sid, ids) in cfg.compatible_outfit_ids.clone().unwrap_or_default() {
        if sid.parse::<u32>().is_err() {
            errors.push(format!("invalid outfit map scene key {sid}"));
        }
        for id in ids {
            if !outfit_ids.contains(&id) {
                errors.push(format!(
                    "compatible_outfit_ids scene {sid} unknown outfit {id}"
                ));
            }
        }
    }
    Ok(
        json!({"ok": errors.is_empty(), "errors": errors, "config": path, "version": cfg.version.unwrap_or_else(|| "unknown".into()), "scene_count": cfg.scenes.len(), "style_count": cfg.styles.len(), "outfit_count": cfg.outfits.len()}),
    )
}

fn status(cli: &Cli) -> Result<Value> {
    ensure_dirs(cli)?;
    let vr = validate_report(&cli.dice_config)?;
    Ok(
        json!({"ok": true, "version": VERSION, "runtime_version": VERSION, "new": image_files(&cli.new_dir).len(), "new_limit": 100, "used": image_files(&cli.used_dir).len(), "k_original": image_files(&cli.k_original).len(), "templates": yaml_files(&cli.prompt_lib.join("templates")).len(), "dice_config": vr, "paths": {"K-original": cli.k_original, "k-selfie-new": cli.new_dir, "k-selfie-used": cli.used_dir, "prompt_lib": cli.prompt_lib, "runtime_dir": cli.runtime_dir}}),
    )
}
fn yaml_files(dir: &Path) -> Vec<PathBuf> {
    if !dir.exists() {
        return vec![];
    };
    let mut xs: Vec<_> = WalkDir::new(dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_path_buf())
        .filter(|p| p.is_file())
        .filter(|p| {
            matches!(
                p.extension().and_then(|s| s.to_str()).unwrap_or(""),
                "yaml" | "yml" | "md"
            )
        })
        .collect();
    xs.sort();
    xs
}

fn value_to_string_list(v: &Value) -> Vec<String> {
    match v {
        Value::Array(a) => a
            .iter()
            .filter_map(|x| x.as_str().map(|s| s.trim().to_string()))
            .filter(|s| !s.is_empty())
            .collect(),
        Value::String(s) => {
            let t = s.trim();
            if t.starts_with('[') && t.ends_with(']') {
                t.trim_start_matches('[')
                    .trim_end_matches(']')
                    .split(',')
                    .map(|x| x.trim().trim_matches('"').trim_matches('\'').to_string())
                    .filter(|x| !x.is_empty())
                    .collect()
            } else if t.is_empty() {
                vec![]
            } else {
                vec![t.to_string()]
            }
        }
        _ => vec![],
    }
}
fn str_list(v: &Value, key: &str) -> Vec<String> {
    v.get(key).map(value_to_string_list).unwrap_or_default()
}
fn parse_template_document(txt: &str, path: &Path) -> Result<(Value, String)> {
    if let Ok(y) = serde_yaml::from_str::<serde_yaml::Value>(txt) {
        return Ok((yaml_value_to_json(y), String::new()));
    }
    let normalized = txt.replace("\r\n", "\n");
    if let Some(stripped) = normalized.strip_prefix("---\n") {
        if let Some(pos) = stripped.find("\n---") {
            let fm = &stripped[..pos];
            let body_start = pos + "\n---".len();
            let body = stripped
                .get(body_start..)
                .unwrap_or("")
                .trim_start_matches('\n')
                .to_string();
            let y: serde_yaml::Value = serde_yaml::from_str(fm)
                .with_context(|| format!("parse frontmatter {}", path.display()))?;
            return Ok((yaml_value_to_json(y), body));
        }
    }
    bail!("parse yaml or frontmatter {}", path.display())
}
fn collect_markdown_fragments(body: &str, out: &mut Vec<String>) {
    for line in body.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("- ") {
            let x = rest.trim().trim_matches('"').trim_matches('\'');
            if x.chars().count() >= 8 {
                out.push(x.to_string());
            }
        }
    }
}
fn collect_strings(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => out.push(s.clone()),
        Value::Array(a) => {
            for x in a {
                collect_strings(x, out);
            }
        }
        Value::Object(m) => {
            for x in m.values() {
                collect_strings(x, out);
            }
        }
        _ => {}
    }
}
fn yaml_value_to_json(v: serde_yaml::Value) -> Value {
    serde_json::to_value(v).unwrap_or(Value::Null)
}

fn compile_templates(cli: &Cli, out: Option<PathBuf>) -> Result<Value> {
    let template_dir = cli.prompt_lib.join("templates");
    let mut templates = vec![];
    let mut skipped: Vec<Value> = vec![];
    for path in yaml_files(&template_dir) {
        let txt = fs::read_to_string(&path)?;
        let (j, body) = match parse_template_document(&txt, &path) {
            Ok(v) => v,
            Err(e) => {
                skipped.push(json!({"path": path, "error": e.to_string()}));
                continue;
            }
        };
        let mut fragments = vec![];
        if let Some(x) = j.get("fragments") {
            collect_strings(x, &mut fragments);
        }
        if let Some(x) = j.get("randomizable") {
            collect_strings(x, &mut fragments);
        }
        collect_markdown_fragments(&body, &mut fragments);
        let mut avoid = str_list(&j, "avoid");
        if let Some(x) = j.get("negative") {
            collect_strings(x, &mut avoid);
        }
        if let Some(x) = j.get("negative_signal").and_then(|n| n.get("avoid")) {
            collect_strings(x, &mut avoid);
        }
        let entry = TemplateEntry {
            path: path.to_string_lossy().to_string(),
            id: j
                .get("id")
                .and_then(|x| x.as_str())
                .unwrap_or_else(|| {
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                })
                .to_string(),
            name: j
                .get("name")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
            template_type: j
                .get("type")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
            scene_tags: str_list(&j, "scene_tags"),
            style_tags: str_list(&j, "style_tags"),
            outfit_tags: str_list(&j, "outfit_tags"),
            mood_tags: str_list(&j, "mood_tags"),
            fragment_texts: fragments
                .into_iter()
                .filter(|s| !s.trim().is_empty())
                .take(24)
                .collect(),
            avoid: avoid
                .into_iter()
                .filter(|s| !s.trim().is_empty())
                .take(24)
                .collect(),
            positive_weight: j
                .get("positive_signal")
                .and_then(|p| p.get("weight"))
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
            negative_weight: j
                .get("negative_signal")
                .and_then(|p| p.get("weight"))
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
        };
        templates.push(entry);
    }
    let index = TemplateIndex {
        schema: "selfiek.template_index.v1".into(),
        version: VERSION.into(),
        generated_at: Utc::now().to_rfc3339(),
        source_dir: template_dir.to_string_lossy().to_string(),
        template_count: templates.len(),
        templates,
    };
    let out_path = out.unwrap_or_else(|| cli.runtime_dir.join("template_index.json"));
    write_json_atomic(&out_path, &serde_json::to_value(&index)?)?;
    Ok(
        json!({"ok": skipped.is_empty(), "out": out_path, "template_count": index.template_count, "skipped_count": skipped.len(), "skipped": skipped, "schema": index.schema, "version": VERSION}),
    )
}

fn load_template_index(cli: &Cli) -> Option<TemplateIndex> {
    read_json_file(&cli.runtime_dir.join("template_index.json")).ok()
}
fn template_score(t: &TemplateEntry, scene: &Scene, style: &Style, outfit: &Outfit) -> i32 {
    let hay_scene = format!("{} {}", scene.name, scene.prompt);
    let hay_style = format!("{} {}", style.name, style.prompt);
    let hay_outfit = format!("{} {}", outfit.name, outfit.prompt);
    let mut score = 0;
    for tag in &t.scene_tags {
        if hay_scene.contains(tag) {
            score += 4;
        }
    }
    for tag in &t.style_tags {
        if hay_style.contains(tag) {
            score += 3;
        }
    }
    for tag in &t.outfit_tags {
        if hay_outfit.contains(tag) {
            score += 2;
        }
    }
    if t.positive_weight.is_some() {
        score += 2;
    }
    if t.negative_weight.is_some() {
        score -= 3;
    }
    score
}

fn choose_template_card(cli: &Cli, scene: &Scene, style: &Style, outfit: &Outfit) -> Option<Value> {
    let index = load_template_index(cli)?;
    let mut scored: Vec<_> = index
        .templates
        .iter()
        .map(|t| (template_score(t, scene, style, outfit), t))
        .filter(|(s, _)| *s > 0)
        .collect();
    scored.sort_by_key(|item| std::cmp::Reverse(item.0));
    let best = scored.first()?.1;
    let fragments: Vec<_> = best.fragment_texts.iter().take(6).cloned().collect();
    Some(
        json!({"schema":"selfiek.prompt_card.v1", "template_id": best.id, "template_path": best.path, "score": scored.first().map(|x| x.0).unwrap_or(0), "fragments": fragments, "avoid": best.avoid.iter().take(6).cloned().collect::<Vec<_>>() }),
    )
}

fn draw(
    cli: &Cli,
    scene_id: Option<u32>,
    style_id: Option<u32>,
    outfit_id: Option<u32>,
    use_templates: bool,
) -> Result<DrawResult> {
    ensure_dirs(cli)?;
    let cfg = dice(cli)?;
    let mut rng = rand::thread_rng();
    let scene = match scene_id {
        Some(id) => cfg
            .scenes
            .iter()
            .find(|s| s.id == id)
            .cloned()
            .ok_or_else(|| anyhow!("scene {id} not found"))?,
        None => cfg
            .scenes
            .choose(&mut rng)
            .cloned()
            .ok_or_else(|| anyhow!("no scenes"))?,
    };
    let style_ids = cfg
        .compatible_style_ids
        .clone()
        .unwrap_or_default()
        .get(&scene.id.to_string())
        .cloned()
        .unwrap_or_else(|| cfg.styles.iter().map(|s| s.id).collect());
    let style = match style_id {
        Some(id) => cfg
            .styles
            .iter()
            .find(|s| s.id == id)
            .cloned()
            .ok_or_else(|| anyhow!("style {id} not found"))?,
        None => {
            let id = *style_ids
                .choose(&mut rng)
                .ok_or_else(|| anyhow!("no compatible styles"))?;
            cfg.styles
                .iter()
                .find(|s| s.id == id)
                .cloned()
                .ok_or_else(|| anyhow!("style {id} missing"))?
        }
    };
    let outfit_ids = cfg
        .compatible_outfit_ids
        .clone()
        .unwrap_or_default()
        .get(&scene.id.to_string())
        .cloned()
        .unwrap_or_else(|| cfg.outfits.iter().map(|o| o.id).collect());
    let outfit = match outfit_id {
        Some(id) => cfg
            .outfits
            .iter()
            .find(|o| o.id == id)
            .cloned()
            .ok_or_else(|| anyhow!("outfit {id} not found"))?,
        None => {
            let id = *outfit_ids
                .choose(&mut rng)
                .ok_or_else(|| anyhow!("no compatible outfits"))?;
            cfg.outfits
                .iter()
                .find(|o| o.id == id)
                .cloned()
                .ok_or_else(|| anyhow!("outfit {id} missing"))?
        }
    };
    let k_images = {
        let imgs = image_files(&cli.k_original);
        if imgs.is_empty() {
            bail!("no K reference images in {}", cli.k_original.display())
        };
        let n = imgs.len().min(3);
        imgs.choose_multiple(&mut rng, n)
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
    };
    let opening = scene
        .openings
        .choose(&mut rng)
        .cloned()
        .ok_or_else(|| anyhow!("scene {} no openings", scene.id))?;
    let film = cfg
        .film_styles
        .choose(&mut rng)
        .cloned()
        .ok_or_else(|| anyhow!("no film styles"))?;
    let lighting = cfg
        .lighting_styles
        .choose(&mut rng)
        .cloned()
        .ok_or_else(|| anyhow!("no lighting styles"))?;
    let prompt_card = if use_templates {
        choose_template_card(cli, &scene, &style, &outfit)
    } else {
        None
    };
    let mut template_fragment = String::new();
    if let Some(card) = &prompt_card {
        if let Some(arr) = card.get("fragments").and_then(|x| x.as_array()) {
            let xs: Vec<_> = arr.iter().filter_map(|x| x.as_str()).take(5).collect();
            if !xs.is_empty() {
                template_fragment = format!("\n【提示词库可迁移片段】{}", xs.join("；"));
            }
        }
    }
    let full_prompt = format!("【主题与风格】一位顶级摄影大师拍摄的生活化人像照片，{}\n【角色描述】参考图拼图里包含同一位角色的多张照片，只提取稳定一致的面部特征和发型气质，生成一个全新独立场景；不要复制参考图的服装、姿势、背景、光线或拍摄角度\n【场景与氛围】{}\n【服装】{}；服装需要与场景自然匹配，生活化、得体，不抢人物和场景的真实感\n【拍摄方式】{}\n【光线】{}{}\n【画质要求】真实自然的皮肤纹理，自然光影，8K超高细节，极致真实质感", film, scene.prompt, outfit.prompt, style.prompt, lighting, template_fragment);
    let mut warnings = vec![];
    if let Some(id) = style_id {
        if !style_ids.contains(&id) {
            warnings.push(format!("style {id} is unusual for scene {}", scene.id));
        }
    }
    if let Some(id) = outfit_id {
        if !outfit_ids.contains(&id) {
            warnings.push(format!("outfit {id} is unusual for scene {}", scene.id));
        }
    }
    Ok(DrawResult {
        ok: true,
        drawn_at: Local::now().to_rfc3339(),
        scene: IdName {
            id: scene.id,
            name: scene.name,
        },
        style: IdName {
            id: style.id,
            name: style.name,
        },
        outfit: IdName {
            id: outfit.id,
            name: outfit.name,
        },
        k_images,
        opening,
        film_style: film,
        lighting,
        scene_prompt: scene.prompt,
        style_prompt: style.prompt,
        outfit_prompt: outfit.prompt,
        compatibility_warning: if warnings.is_empty() {
            None
        } else {
            Some(warnings.join("; "))
        },
        full_prompt,
        prompt_card,
    })
}

fn build_reference_collage(cli: &Cli, paths: &[String]) -> Result<PathBuf> {
    let panel = 768u32;
    let gap = 24u32;
    let mut canvases: Vec<RgbImage> = vec![];
    for p in paths {
        let img = image::open(p)
            .with_context(|| format!("open image {p}"))?
            .to_rgb8();
        let (w, h) = img.dimensions();
        let scale = (panel as f32 / w as f32).min(panel as f32 / h as f32);
        let nw = (w as f32 * scale).round() as u32;
        let nh = (h as f32 * scale).round() as u32;
        let resized = image::imageops::resize(&img, nw, nh, image::imageops::FilterType::Lanczos3);
        let mut canvas = ImageBuffer::from_pixel(panel, panel, Rgb([245, 245, 245]));
        let x = (panel - nw) / 2;
        let y = (panel - nh) / 2;
        canvas.copy_from(&resized, x, y)?;
        canvases.push(canvas);
    }
    let width = panel * canvases.len() as u32 + gap * (canvases.len().saturating_sub(1) as u32);
    let mut out = ImageBuffer::from_pixel(width, panel, Rgb([245, 245, 245]));
    for (i, c) in canvases.iter().enumerate() {
        out.copy_from(c, i as u32 * (panel + gap), 0)?;
    }
    let tmp = std::env::temp_dir().join(format!(
        "selfiek_refs_{}.jpg",
        Utc::now().timestamp_micros()
    ));
    DynamicImage::ImageRgb8(out).save_with_format(&tmp, image::ImageFormat::Jpeg)?;
    let _ = cli;
    Ok(tmp)
}

#[allow(clippy::too_many_arguments)]
fn generate(
    cli: &Cli,
    scene: Option<u32>,
    style: Option<u32>,
    outfit: Option<u32>,
    use_templates: bool,
    dry_run: bool,
    quiet: bool,
    out_dir: Option<PathBuf>,
) -> Result<Value> {
    ensure_dirs(cli)?;
    let draw = draw(cli, scene, style, outfit, use_templates)?;
    let collage = build_reference_collage(cli, &draw.k_images)?;
    let full_prompt = format!(
        "{}\n\n{}{}",
        REF_IMAGE_PREFIX, draw.full_prompt, NEGATIVE_SUFFIX
    );
    if dry_run {
        return Ok(
            json!({"ok": true, "dry_run": true, "draw": draw, "reference_collage": collage, "prompt": full_prompt}),
        );
    }
    let out_dir = out_dir.unwrap_or_else(|| cli.new_dir.clone());
    fs::create_dir_all(&out_dir)?;
    let base = format!(
        "{}_s{:02}_st{:02}_o{:02}_{:x}",
        Local::now().format("%Y%m%d_%H%M%S"),
        draw.scene.id,
        draw.style.id,
        draw.outfit.id,
        md5::compute(draw.k_images.join("|"))
    );
    let base = base[..base.len().min(48)].to_string();
    let final_png = out_dir.join(format!("{base}.png"));
    let final_json = out_dir.join(format!("{base}.json"));
    let tmp_png = out_dir.join(format!(".{base}.tmp.png"));
    if !quiet {
        eprintln!("[selfiek] generating {}", final_png.display());
    }
    let gen_lock_path = cli.runtime_dir.join(".selfiek.generation.lock");
    let gen_lock = File::create(gen_lock_path)?;
    gen_lock.lock_exclusive()?;
    let output = Command::new(&cli.cdper_bin)
        .arg("generate")
        .arg("--timeout-sec")
        .arg("300")
        .arg("--image")
        .arg(&collage)
        .arg("--prompt")
        .arg(&full_prompt)
        .arg("--out")
        .arg(&tmp_png)
        .output()
        .with_context(|| format!("run {}", cli.cdper_bin.display()))?;
    if !output.status.success() {
        let _ = fs::remove_file(&tmp_png);
        bail!(
            "cdper failed: status={:?}; stderr={}; stdout={}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
                .chars()
                .take(2000)
                .collect::<String>(),
            String::from_utf8_lossy(&output.stdout)
                .chars()
                .take(2000)
                .collect::<String>()
        );
    }
    if !tmp_png.exists() || fs::metadata(&tmp_png)?.len() < 1024 {
        bail!("output file missing or too small");
    }
    fs::rename(&tmp_png, &final_png)?;
    let cdper = parse_last_json(&String::from_utf8_lossy(&output.stdout)).unwrap_or_else(|| json!({"raw_stdout": String::from_utf8_lossy(&output.stdout).chars().rev().take(2000).collect::<String>()}));
    let metadata = json!({"ok":true,"created_at":Local::now().to_rfc3339(),"generator":"selfiek","selfiek_version":VERSION,"image":final_png,"opening":draw.opening,"scene":draw.scene,"style":draw.style,"outfit":draw.outfit,"k_images":draw.k_images,"reference_collage":collage,"film_style":draw.film_style,"lighting":draw.lighting,"prompt":full_prompt,"prompt_card":draw.prompt_card,"cdper":cdper});
    write_json_atomic(&final_json, &metadata)?;
    Ok(
        json!({"ok":true,"image":final_png,"metadata":final_json,"opening":metadata["opening"],"source":"generated","generator":"selfiek","version":VERSION}),
    )
}

fn parse_last_json(s: &str) -> Option<Value> {
    if let Ok(v) = serde_json::from_str(s) {
        return Some(v);
    };
    for (i, ch) in s.char_indices().rev() {
        if ch == '{' {
            if let Ok(v) = serde_json::from_str(&s[i..]) {
                return Some(v);
            }
        }
    }
    None
}
fn write_json_atomic(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension(format!(
        "{}tmp",
        path.extension().and_then(|s| s.to_str()).unwrap_or("json.")
    ));
    let mut f = File::create(&tmp)?;
    f.write_all(serde_json::to_string_pretty(value)?.as_bytes())?;
    f.sync_all().ok();
    fs::rename(tmp, path)?;
    Ok(())
}

fn consume_stock(cli: &Cli) -> Result<Value> {
    ensure_dirs(cli)?;
    let lock_path = cli.runtime_dir.join(".selfiek.consume.lock");
    let lock = File::create(lock_path)?;
    lock.lock_exclusive()?;
    let files = image_files(&cli.new_dir);
    if files.is_empty() {
        bail!("k-selfie-new is empty");
    }
    let oldest = &files[0];
    let meta_src = oldest.with_extension("json");
    let mut meta: Value = read_json_file(&meta_src)
        .with_context(|| format!("metadata required for {}", oldest.display()))?;
    let opening = meta
        .get("opening")
        .and_then(|x| x.as_str())
        .ok_or_else(|| anyhow!("metadata/opening missing; refusing to re-roll opening"))?
        .to_string();
    let mut dst_img = cli.used_dir.join(oldest.file_name().unwrap());
    let mut dst_json = cli.used_dir.join(meta_src.file_name().unwrap());
    if dst_img.exists() {
        let stem = oldest
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("selfie");
        let suffix = Utc::now().timestamp();
        dst_img = cli.used_dir.join(format!("{stem}_{suffix}.png"));
        dst_json = cli.used_dir.join(format!("{stem}_{suffix}.json"));
    }
    meta["consumed_at"] = json!(Local::now().to_rfc3339());
    meta["image"] = json!(dst_img.to_string_lossy().to_string());
    write_json_atomic(&meta_src, &meta)?;
    fs::rename(oldest, &dst_img)?;
    fs::rename(&meta_src, &dst_json)?;
    let remaining = image_files(&cli.new_dir).len();
    lock.unlock()?;
    Ok(
        json!({"ok":true,"image":dst_img,"metadata":dst_json,"opening":opening,"source":"stock","remaining":remaining,"generator":"selfiek"}),
    )
}

fn next(cli: &Cli, use_templates: bool) -> Result<Value> {
    match consume_stock(cli) {
        Ok(v) => Ok(v),
        Err(_) => {
            let gen = generate(
                cli,
                None,
                None,
                None,
                use_templates,
                false,
                true,
                Some(cli.new_dir.clone()),
            )?;
            if !gen.get("ok").and_then(|x| x.as_bool()).unwrap_or(false) {
                return Ok(
                    json!({"ok": false, "error": "stock empty and emergency generation failed", "generation": gen}),
                );
            }
            let mut v = consume_stock(cli)?;
            v["source"] = json!("emergency_generated");
            Ok(v)
        }
    }
}

fn produce(cli: &Cli, use_templates: bool, quiet: bool, dry_run: bool) -> Result<Value> {
    ensure_dirs(cli)?;
    let lock_path = cli.runtime_dir.join(".selfiek.produce.lock");
    let lock = File::create(lock_path)?;
    if lock.try_lock_exclusive().is_err() {
        return Ok(json!({"ok": true, "skipped": "producer already running"}));
    }
    let stock = image_files(&cli.new_dir).len();
    if stock >= 100 {
        lock.unlock()?;
        return Ok(json!({"ok": true, "skipped": "full", "stock": stock}));
    }
    let state_path = cli.runtime_dir.join(".selfiek.produce_state.json");
    let mut state: Value =
        read_json_file(&state_path).unwrap_or_else(|_| json!({"batch_count":0,"pause_until":0}));
    let now = Utc::now().timestamp();
    let pause_until = state
        .get("pause_until")
        .and_then(|x| x.as_i64())
        .unwrap_or(0);
    if now < pause_until {
        lock.unlock()?;
        return Ok(
            json!({"ok": true, "skipped": "paused", "stock": stock, "pause_remaining_sec": pause_until - now, "state": state}),
        );
    }
    if dry_run {
        lock.unlock()?;
        return Ok(
            json!({"ok": true, "dry_run": true, "would_generate": true, "stock": stock, "use_templates": use_templates}),
        );
    }
    let result = generate(
        cli,
        None,
        None,
        None,
        use_templates,
        false,
        quiet,
        Some(cli.new_dir.clone()),
    )?;
    if !result.get("ok").and_then(|x| x.as_bool()).unwrap_or(false) {
        lock.unlock()?;
        return Ok(result);
    }
    let stock_after = image_files(&cli.new_dir).len();
    let mut batch = state
        .get("batch_count")
        .and_then(|x| x.as_i64())
        .unwrap_or(0)
        + 1;
    state["last_generated_at"] = json!(Local::now().to_rfc3339());
    state["last_image"] = result.get("image").cloned().unwrap_or(Value::Null);
    if batch >= 3 {
        batch = 0;
        let pause = if stock_after < 20 { 1800 } else { 7200 };
        state["pause_until"] = json!(now + pause);
        state["pause_reason"] = json!(if stock_after < 20 {
            "emergency_low_stock"
        } else {
            "normal_batch_limit"
        });
    }
    state["batch_count"] = json!(batch);
    write_json_atomic(&state_path, &state)?;
    lock.unlock()?;
    Ok(
        json!({"ok": true, "image": result.get("image"), "metadata": result.get("metadata"), "stock": stock_after, "state": state, "generator":"selfiek"}),
    )
}

fn cleanup_used(cli: &Cli, days: i64) -> Result<Value> {
    ensure_dirs(cli)?;
    let cutoff = Utc::now() - Duration::days(days);
    let mut removed = 0;
    for p in image_files(&cli.used_dir) {
        let meta = fs::metadata(&p)?;
        let modified: DateTime<Utc> = meta.modified()?.into();
        if modified < cutoff {
            let j = p.with_extension("json");
            let _ = fs::remove_file(&p);
            let _ = fs::remove_file(&j);
            removed += 1;
        }
    }
    Ok(json!({"ok":true,"removed":removed,"retention_days":days}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_frontmatter_with_markdown_body() {
        let txt = "---\ntype: selfie-prompt-template\nid: demo\nscene_tags: \"[海边, 沙滩]\"\n---\n\n# Title\n- 柔和逆光\n- 手机抓拍";
        let (j, body) = parse_template_document(txt, Path::new("demo.yaml")).unwrap();
        assert_eq!(j.get("id").and_then(|x| x.as_str()), Some("demo"));
        assert!(body.contains("手机抓拍"));
        assert_eq!(
            str_list(&j, "scene_tags"),
            vec!["海边".to_string(), "沙滩".to_string()]
        );
    }

    #[test]
    fn collects_markdown_fragments() {
        let mut out = Vec::new();
        collect_markdown_fragments("# Demo\n- 彩色舞台灯扫过脸侧\n- 朋友突然按下快门", &mut out);
        assert_eq!(out.len(), 2);
        assert!(out[0].contains("舞台灯"));
    }
}
