use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, bail, Context};
use regex::{Captures, Regex};

use crate::{
    config::{ImageOptimizeMode, ImagesConf},
    project::Project,
    slide::Slide,
};

#[derive(Debug, Clone)]
pub struct OptimizeOptions {
    pub dry_run: bool,
    pub force: bool,
}

pub enum ImageRewriteMode {
    CacheRelativeToMarkdown,
    PublicAssets { base_dir: PathBuf },
}

#[derive(Debug, Clone)]
pub struct ImageRef {
    original: String,
    path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct OptimizedImage {
    original: String,
    source_path: PathBuf,
    cache_path: PathBuf,
    optimized: bool,
    cached: bool,
}

#[derive(Debug, Clone, Default)]
pub struct OptimizeReport {
    pub images: Vec<OptimizedImage>,
    pub dry_run: bool,
}

impl OptimizeReport {
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    pub fn log(&self, slide: &Slide) {
        if self.is_empty() {
            log::info!(
                "optimize images: {} ... no images",
                slide.dir.to_string_lossy()
            );
            return;
        }

        let optimized = self.images.iter().filter(|image| image.optimized).count();
        let cached = self.images.iter().filter(|image| image.cached).count();
        log::info!(
            "optimize images: {} ... {} image(s), {} optimized, {} cached",
            slide.dir.to_string_lossy(),
            self.images.len(),
            optimized,
            cached
        );
    }
}

impl OptimizedImage {
    pub fn cache_path(&self) -> &Path {
        &self.cache_path
    }
}

pub fn optimize_slide_images(
    project: &Project,
    slide: &Slide,
    options: &OptimizeOptions,
) -> anyhow::Result<OptimizeReport> {
    let markdown_path = slide.dir.join("slide.md");
    let contents = fs::read_to_string(&markdown_path)?;
    let refs = collect_image_refs(slide, &contents)?;
    optimize_image_refs(project, slide, refs, options)
}

pub fn prepare_optimized_markdown(
    project: &Project,
    slide: &Slide,
    contents: &str,
    options: &OptimizeOptions,
    rewrite_mode: ImageRewriteMode,
) -> anyhow::Result<(String, OptimizeReport)> {
    if !project.conf.images.enabled {
        return Ok((contents.to_string(), OptimizeReport::default()));
    }

    let refs = collect_image_refs(slide, contents)?;
    let report = optimize_image_refs(project, slide, refs, options)?;
    let rewritten = rewrite_image_refs(contents, &slide.dir, &report.images, rewrite_mode)?;

    Ok((rewritten, report))
}

pub fn clean_image_cache(project: &Project) -> anyhow::Result<PathBuf> {
    let cache_dir = project.root_dir.join(&project.conf.images.cache_dir);
    let public_images_dir = project
        .root_dir
        .join(&project.conf.output_dir)
        .join("images");

    if cache_dir == public_images_dir || cache_dir.starts_with(&public_images_dir) {
        bail!(
            "refusing to remove public image output directory as cache: {}",
            cache_dir.to_string_lossy()
        );
    }

    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)?;
    }

    Ok(cache_dir)
}

fn collect_image_refs(slide: &Slide, contents: &str) -> anyhow::Result<Vec<ImageRef>> {
    let markdown = Regex::new(r#"!\[[^\]]*\]\((?P<url>[^)\s]+)(?:\s+"[^"]*")?\)"#)?;
    let html = Regex::new(r#"<img\b[^>]*\bsrc=["'](?P<url>[^"']+)["'][^>]*>"#)?;
    let mut refs = Vec::new();

    for caps in markdown.captures_iter(contents) {
        push_image_ref(slide, &mut refs, &caps)?;
    }
    for caps in html.captures_iter(contents) {
        push_image_ref(slide, &mut refs, &caps)?;
    }

    refs.sort_by(|a, b| a.original.cmp(&b.original));
    refs.dedup_by(|a, b| a.original == b.original);
    Ok(refs)
}

fn push_image_ref(slide: &Slide, refs: &mut Vec<ImageRef>, caps: &Captures) -> anyhow::Result<()> {
    let Some(url) = caps.name("url").map(|m| m.as_str()) else {
        return Ok(());
    };

    if should_skip_url(url) {
        return Ok(());
    }

    let path_part = url.split(['?', '#']).next().unwrap_or(url);
    let path = slide.dir.join(path_part);
    if !path.exists() || !path.is_file() {
        return Ok(());
    }

    refs.push(ImageRef {
        original: url.to_string(),
        path,
    });

    Ok(())
}

fn should_skip_url(url: &str) -> bool {
    url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("data:")
        || url.starts_with('/')
        || url.starts_with('#')
}

fn optimize_image_refs(
    project: &Project,
    slide: &Slide,
    refs: Vec<ImageRef>,
    options: &OptimizeOptions,
) -> anyhow::Result<OptimizeReport> {
    let cache_dir = project.root_dir.join(&project.conf.images.cache_dir);
    if !options.dry_run {
        fs::create_dir_all(&cache_dir)?;
    }

    let mut images = Vec::new();
    for image_ref in refs {
        let extension = image_ref
            .path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let cache_path = cache_dir.join(format!(
            "{}.{}",
            cache_key(&image_ref.path, &project.conf.images, &extension)?,
            extension
        ));

        let cached = cache_path.exists() && !options.force;
        let mut optimized = false;

        if !cached && !options.dry_run {
            optimized = optimize_one(&project.conf.images, &image_ref.path, &cache_path)
                .with_context(|| image_ref.path.to_string_lossy().to_string())?;
        }

        images.push(OptimizedImage {
            original: image_ref.original,
            source_path: image_ref.path,
            cache_path,
            optimized,
            cached,
        });
    }

    let report = OptimizeReport {
        images,
        dry_run: options.dry_run,
    };
    report.log(slide);
    Ok(report)
}

fn cache_key(path: &Path, conf: &ImagesConf, extension: &str) -> anyhow::Result<String> {
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified().ok();
    let mut hasher = DefaultHasher::new();

    path.canonicalize()?.hash(&mut hasher);
    metadata.len().hash(&mut hasher);
    modified.hash(&mut hasher);
    extension.hash(&mut hasher);
    conf.mode.hash(&mut hasher);
    conf.strip_metadata.hash(&mut hasher);
    conf.png.level.hash(&mut hasher);
    conf.jpeg.quality.hash(&mut hasher);

    Ok(format!("{:016x}", hasher.finish()))
}

fn optimize_one(conf: &ImagesConf, input: &Path, output: &Path) -> anyhow::Result<bool> {
    let extension = input
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match extension.as_str() {
        "png" if conf.png.enabled => run_oxipng(conf, input, output),
        "jpg" | "jpeg" if conf.jpeg.enabled => run_jpegoptim(conf, input, output),
        "svg" if conf.svg.enabled => run_svgo(conf, input, output),
        _ => {
            fs::copy(input, output)?;
            Ok(false)
        }
    }
}

fn run_oxipng(conf: &ImagesConf, input: &Path, output: &Path) -> anyhow::Result<bool> {
    if !command_exists(&conf.png.tool) {
        return missing_tool(conf, &conf.png.tool, input, output);
    }

    let mut cmd = Command::new(&conf.png.tool);
    cmd.arg("-o")
        .arg(conf.png.level.to_string())
        .arg("--out")
        .arg(output);

    if conf.strip_metadata {
        cmd.arg("--strip").arg("safe");
    }

    cmd.arg(input);
    run_optimizer(cmd)?;
    Ok(true)
}

fn run_jpegoptim(conf: &ImagesConf, input: &Path, output: &Path) -> anyhow::Result<bool> {
    if !command_exists(&conf.jpeg.tool) {
        return missing_tool(conf, &conf.jpeg.tool, input, output);
    }

    fs::copy(input, output)?;

    let mut cmd = Command::new(&conf.jpeg.tool);
    if conf.strip_metadata {
        cmd.arg("--strip-all");
    }
    if conf.mode == ImageOptimizeMode::Lossy {
        cmd.arg(format!("--max={}", conf.jpeg.quality));
    }
    cmd.arg(output);
    run_optimizer(cmd)?;
    Ok(true)
}

fn run_svgo(conf: &ImagesConf, input: &Path, output: &Path) -> anyhow::Result<bool> {
    if !command_exists(&conf.svg.tool) {
        return missing_tool(conf, &conf.svg.tool, input, output);
    }

    let mut cmd = Command::new(&conf.svg.tool);
    cmd.arg(input).arg("-o").arg(output);
    run_optimizer(cmd)?;
    Ok(true)
}

fn missing_tool(
    conf: &ImagesConf,
    tool: &str,
    input: &Path,
    output: &Path,
) -> anyhow::Result<bool> {
    if conf.fail_on_missing_tool {
        return Err(anyhow!("image optimizer is missing: {tool}"));
    }

    log::warn!(
        "image optimizer is missing: {}; copy without optimization: {}",
        tool,
        input.to_string_lossy()
    );
    fs::copy(input, output)?;
    Ok(false)
}

fn command_exists(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn run_optimizer(mut command: Command) -> anyhow::Result<()> {
    let output = command.output()?;
    if output.status.success() {
        return Ok(());
    }

    Err(anyhow!(
        "optimizer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn rewrite_image_refs(
    contents: &str,
    markdown_dir: &Path,
    images: &[OptimizedImage],
    rewrite_mode: ImageRewriteMode,
) -> anyhow::Result<String> {
    let mut rewritten = contents.to_string();

    for image in images {
        let relative = match &rewrite_mode {
            ImageRewriteMode::CacheRelativeToMarkdown => {
                relative_path(markdown_dir, &image.cache_path)
            }
            ImageRewriteMode::PublicAssets { base_dir } => {
                let Some(file_name) = image.cache_path.file_name() else {
                    continue;
                };
                base_dir.join(file_name)
            }
        };
        let replacement = preserve_suffix(&image.original, &relative);
        rewritten = rewritten.replace(&image.original, &replacement);
    }

    Ok(rewritten)
}

fn preserve_suffix(original: &str, replacement_path: &Path) -> String {
    let suffix = original
        .find(['?', '#'])
        .map(|index| &original[index..])
        .unwrap_or_default();

    format!("{}{}", replacement_path.to_string_lossy(), suffix)
}

pub fn relative_path(from_dir: &Path, to_path: &Path) -> PathBuf {
    let from_components: Vec<_> = from_dir.components().collect();
    let to_components: Vec<_> = to_path.components().collect();
    let common_len = from_components
        .iter()
        .zip(to_components.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let mut relative = PathBuf::new();
    for _ in common_len..from_components.len() {
        relative.push("..");
    }
    for component in &to_components[common_len..] {
        relative.push(component.as_os_str());
    }

    if relative.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        relative
    }
}

pub fn print_report(report: &OptimizeReport, project: &Project) {
    if report.is_empty() {
        println!("No images found.");
        return;
    }

    for image in &report.images {
        let action = if image.cached {
            "cached"
        } else if report.dry_run {
            "would optimize"
        } else if image.optimized {
            "optimized"
        } else {
            "copied"
        };
        println!(
            "{} -> {} ({action})",
            display_path(project, &image.source_path),
            display_path(project, &image.cache_path)
        );
    }
}

fn display_path(project: &Project, path: &Path) -> String {
    path.strip_prefix(&project.root_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

#[cfg(test)]
mod tests {
    use crate::{
        config::{ImagesConf, ProjectConf},
        project::Project,
    };

    use super::clean_image_cache;

    #[test]
    fn clean_image_cache_refuses_public_optimized_images() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let public_image = root
            .join("output")
            .join("images")
            .join("optimized")
            .join("example.png");
        std::fs::create_dir_all(public_image.parent().unwrap()).unwrap();
        std::fs::write(&public_image, "fake").unwrap();

        let conf = ProjectConf {
            output_dir: "output".to_string(),
            images: ImagesConf {
                cache_dir: "output/images/optimized".to_string(),
                ..ImagesConf::default()
            },
            ..ProjectConf::default()
        };
        let project = Project {
            root_dir: root.to_path_buf(),
            conf,
            slides: vec![],
        };

        let error = clean_image_cache(&project).unwrap_err();

        assert!(error
            .to_string()
            .contains("refusing to remove public image output directory"));
        assert!(public_image.exists());
    }
}
