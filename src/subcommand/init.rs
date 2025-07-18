use std::fs;
use std::path::Path;

use crate::config::ProjectConf;

/// Initialize a new slide project at the given root directory.
/// This function creates the necessary directory structure and a default `config.toml`.
///
/// # Arguments
/// * `root_dir` - The path to the root directory where the project should be initialized.
pub fn init(root_dir: &Path) -> anyhow::Result<()> {
    log::info!(
        "Initializing new slide project at: {}",
        root_dir.to_string_lossy()
    );

    // Create .marp/themes directory
    let marp_themes_dir = root_dir.join(".marp").join("themes");
    if marp_themes_dir.exists() {
        anyhow::bail!(
            "Directory already exists: {}",
            marp_themes_dir.to_string_lossy()
        );
    }
    log::info!("Creating directory: {}", marp_themes_dir.to_string_lossy());
    fs::create_dir_all(&marp_themes_dir)?; // Create all necessary parent directories
    fs::write(marp_themes_dir.join(".gitkeep"), "")?; // Create a .gitkeep file

    // Create src directory
    let src_dir = root_dir.join("src");
    if src_dir.exists() {
        anyhow::bail!("Directory already exists: {}", src_dir.to_string_lossy());
    }
    log::info!("Creating directory: {}", src_dir.to_string_lossy());
    fs::create_dir_all(&src_dir)?; // Create all necessary parent directories
    fs::write(src_dir.join(".gitkeep"), "")?; // Create a .gitkeep file

    // Create config.toml
    let config_path = root_dir.join("config.toml");
    if config_path.exists() {
        anyhow::bail!("File already exists: {}", config_path.to_string_lossy());
    }

    // Create a default ProjectConf and serialize it to a TOML string
    let default_config = ProjectConf::default();
    let config_content = toml::to_string_pretty(&default_config)?; // Use toml::to_string_pretty for formatted output

    log::info!("Creating config file: {}", config_path.to_string_lossy());
    fs::write(&config_path, config_content)?; // Write the config content to the file

    log::info!("Project initialized successfully!");
    Ok(())
}

// Test code
#[cfg(test)]
mod test_init {
    use super::*;
    use tempfile::tempdir; // Crate for creating temporary directories for testing

    #[test]
    fn test_init_project() {
        let tmp_dir = tempdir().unwrap();
        let project_root = tmp_dir.path().join("test_project");

        // Execute the init command
        init(&project_root).unwrap();

        // Verify that the expected directories and files exist
        assert!(project_root.exists());
        assert!(project_root.join(".marp").exists());
        assert!(project_root.join(".marp").join("themes").exists());
        assert!(project_root
            .join(".marp")
            .join("themes")
            .join(".gitkeep")
            .exists());
        assert!(project_root.join("src").exists());
        assert!(project_root.join("src").join(".gitkeep").exists());
        assert!(project_root.join("config.toml").exists());

        // Read the content of config.toml and verify it can be parsed
        let config_content = fs::read_to_string(project_root.join("config.toml")).unwrap();
        let config: ProjectConf = toml::from_str(&config_content).unwrap();

        assert_eq!(config.name, "my-slide-project");
        assert_eq!(config.author, "Your Name");
        assert_eq!(config.base_url, "https://example.com/");
        assert_eq!(config.output_dir, "output");
        assert_eq!(config.template.slide, "");
        assert_eq!(config.template.index, "");
        assert_eq!(config.template.suffix, "");
        assert_eq!(config.build.theme_dir, ".marp/themes");
        assert_eq!(config.build.marp_binary, "marp");

        // Verify error when trying to initialize a project in an already existing directory
        let result = init(&project_root);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            format!(
                "Directory already exists: {}",
                project_root.join(".marp").join("themes").to_string_lossy()
            )
        );
    }
}
