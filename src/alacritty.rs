use dirs::config_dir;
use serde::{Deserialize, Serialize};

use crate::error::{AlthemerError, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlacrittyConfig {
    #[serde(default)]
    pub general: GeneralConfig,

    #[serde(flatten)]
    pub other: toml::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneralConfig {
    #[serde(default)]
    pub import: Vec<String>,
}

/// Environment variable holding the path to the alacritty config file.
pub const ALACRITTY_CONFIG_ENV: &str = "ALTHEMER_ALACRITTY_CONFIG";

pub fn get_alacritty_config_dir() -> Result<PathBuf> {
    config_dir()
        .map(|p| p.join("alacritty"))
        .ok_or_else(|| AlthemerError::ConfigNotFound(PathBuf::from("~/.config/alacritty")))
}

fn expand_tilde(path: &Path) -> PathBuf {
    if path.starts_with("~") {
        return PathBuf::from(shellexpand::tilde(&path.display().to_string()).as_ref());
    }

    path.to_path_buf()
}

/// Resolves the alacritty config path, preferring (in order) the CLI flag, the
/// environment, althemer's config file and finally the default location.
fn resolve_alacritty_config_path(
    cli_path: Option<&Path>,
    env_path: Option<&Path>,
    config_path: Option<&Path>,
) -> Result<PathBuf> {
    match cli_path.or(env_path).or(config_path) {
        Some(path) => Ok(expand_tilde(path)),
        None => Ok(get_alacritty_config_dir()?.join("alacritty.toml")),
    }
}

/// Same as [`resolve_alacritty_config_path`], reading the environment itself.
pub fn get_alacritty_config_path(
    cli_path: Option<&Path>,
    config_path: Option<&Path>,
) -> Result<PathBuf> {
    let env_path = std::env::var_os(ALACRITTY_CONFIG_ENV).map(PathBuf::from);

    resolve_alacritty_config_path(cli_path, env_path.as_deref(), config_path)
}

pub fn read_config(path: &Path) -> Result<AlacrittyConfig> {
    let content = std::fs::read_to_string(path)?;
    let config = toml::from_str::<AlacrittyConfig>(&content)?;
    Ok(config)
}

pub fn write_config(path: &Path, config: &AlacrittyConfig) -> Result<()> {
    let content = toml::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub fn get_themes_dir(custom_path: Option<&Path>, alacritty_config: &Path) -> Result<PathBuf> {
    if let Some(path) = custom_path {
        let themes_dir = expand_tilde(path);

        if !themes_dir.exists() {
            return Err(AlthemerError::ThemesDirNotFound(themes_dir));
        }

        return Ok(themes_dir);
    }

    let alacritty_dir = match alacritty_config.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => get_alacritty_config_dir()?,
    };
    let themes_dir = alacritty_dir.join("themes");
    if !themes_dir.exists() {
        return Err(AlthemerError::ThemesDirNotFound(themes_dir));
    }

    Ok(themes_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_path_takes_precedence() {
        let path = resolve_alacritty_config_path(
            Some(Path::new("/cli/alacritty.toml")),
            Some(Path::new("/env/alacritty.toml")),
            Some(Path::new("/config/alacritty.toml")),
        )
        .unwrap();

        assert_eq!(path, PathBuf::from("/cli/alacritty.toml"));
    }

    #[test]
    fn env_path_takes_precedence_over_config() {
        let path = resolve_alacritty_config_path(
            None,
            Some(Path::new("/env/alacritty.toml")),
            Some(Path::new("/config/alacritty.toml")),
        )
        .unwrap();

        assert_eq!(path, PathBuf::from("/env/alacritty.toml"));
    }

    #[test]
    fn config_path_is_used_when_nothing_else_is_set() {
        let path =
            resolve_alacritty_config_path(None, None, Some(Path::new("/config/alacritty.toml")))
                .unwrap();

        assert_eq!(path, PathBuf::from("/config/alacritty.toml"));
    }

    #[test]
    fn falls_back_to_the_default_location() {
        let path = resolve_alacritty_config_path(None, None, None).unwrap();

        assert_eq!(
            path,
            get_alacritty_config_dir().unwrap().join("alacritty.toml")
        );
    }

    #[test]
    fn resolved_path_is_tilde_expanded() {
        let path =
            resolve_alacritty_config_path(Some(Path::new("~/alacritty.toml")), None, None).unwrap();

        assert!(!path.starts_with("~"));
        assert!(path.ends_with("alacritty.toml"));
    }

    #[test]
    fn themes_dir_defaults_next_to_the_alacritty_config() {
        let dir = tempfile::tempdir().unwrap();
        let themes_dir = dir.path().join("themes");
        std::fs::create_dir(&themes_dir).unwrap();

        let resolved =
            get_themes_dir(None, &dir.path().join("alacritty.toml")).expect("Should resolve");

        assert_eq!(resolved, themes_dir);
    }
}
