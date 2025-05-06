use std::path::PathBuf;

use anyhow::bail;

use crate::slide::Slide;

/// 画像ファイルをコピーする
pub fn copy_images(slide: &Slide, target_images_dir: &PathBuf) -> anyhow::Result<()> {
    let slide_images_dir = slide.image_dir();

    // 画像ディレクトリが存在しない場合はスキップ
    if !slide_images_dir.exists() {
        return Ok(());
    }

    // 画像ファイルのコピー
    let images = std::fs::read_dir(&slide_images_dir)?;

    for image in images.filter_map(|e| e.ok()) {
        let path = image.path();
        let file_name = path.file_name().unwrap();
        // 隠しファイルはスキップ
        if file_name.to_string_lossy().starts_with(".") {
            continue;
        }

        let save_path = target_images_dir.join(file_name);
        if save_path.exists() {
            bail!("The image file already exists: {:?}", save_path);
        }

        std::fs::copy(&path, &save_path)?;
    }

    Ok(())
}
