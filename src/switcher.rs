use std::{io::IsTerminal, path::Path};

use crate::{
    alacritty::{AlacrittyConfig, GeneralConfig, read_config, write_config},
    config::AlthemerConfig,
    error::{AlthemerError, Result},
    picker::pick_theme,
    themes::{Theme, get_current_theme_path, get_theme_by_name, list_themes},
};

/// Switches the active Alacritty theme by updating the config file.
pub fn switch_theme(
    name: &str,
    custom_theme_path: Option<&Path>,
    alacritty_config: &Path,
) -> Result<Theme> {
    let theme = get_theme_by_name(name, custom_theme_path, alacritty_config)?;

    if !alacritty_config.exists() {
        return Err(AlthemerError::ConfigNotFound(
            alacritty_config.to_path_buf(),
        ));
    }

    let config = read_config(alacritty_config)?;

    let new_config = AlacrittyConfig {
        general: GeneralConfig {
            import: vec![theme.path.to_string_lossy().into_owned()],
        },
        other: config.other,
    };

    write_config(alacritty_config, &new_config)?;

    Ok(theme)
}

/// Selects a theme from the list and switches to it
pub fn select_theme(
    custom_theme_path: Option<&Path>,
    config: &AlthemerConfig,
    alacritty_config: &Path,
) -> Result<Theme> {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err(AlthemerError::NoTerminal);
    }

    let themes = list_themes(custom_theme_path, alacritty_config)?;
    let current_path = get_current_theme_path(alacritty_config)?;
    let current = current_path
        .as_ref()
        .and_then(|p| themes.iter().find(|t| &t.path == p));

    match pick_theme(&themes, current, config)? {
        Some(theme) => Ok(theme),
        None => Err(AlthemerError::InteractiveError(
            "No theme selected".to_string(),
        )),
    }
}
