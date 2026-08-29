use std::fs;
use std::path::PathBuf;
use directories::{ProjectDirs, BaseDirs};

pub fn make_logging_folder() -> anyhow::Result<()> {
    let log_dir = get_platform_log_dir()?;

    if !log_dir.exists() {
        fs::create_dir_all(&log_dir)?;
    }

    Ok(())
}

pub fn get_platform_log_dir() -> anyhow::Result<PathBuf> {
    if cfg!(target_os = "linux") {
        if let Some(base_dirs) = BaseDirs::new() {
            return Ok(base_dirs.home_dir().join(".argon"));
        }
        anyhow::bail!("Could not resolve Linux home directory");
    }
    if let Some(proj_dirs) = ProjectDirs::from("com", "MyCompany", "MyApp") {
        return Ok(proj_dirs.config_dir().to_path_buf());
    }

    anyhow::bail!("Logging folder location resolution failed");
}
