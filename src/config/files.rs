use super::*;

impl Config {
    pub(crate) fn write_default_config(cli: &Cli, matches: &ArgMatches) -> Result<PathBuf> {
        let dir = if let Some(Some(ref path)) = cli.init_config {
            resolve_config_dir(path)?
        } else if let Some(ref config_file) = cli.config_file
            && arg_has_user_value(matches, "config_file")
        {
            resolve_config_dir(config_file)?
        } else if let Some(env_path) = std::env::var_os(CONFIG_FILE_ENV)
            && !env_path.is_empty()
        {
            resolve_config_dir(Path::new(&env_path))?
        } else {
            default_config_dir()
                .ok_or_else(|| anyhow::anyhow!("Unable to determine user config directory"))?
        };

        let path = dir.join(DEFAULT_CONFIG_FILE_NAME);

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                anyhow::bail!("Config file already exists: {}", path.display());
            }
            Err(error) => return Err(error.into()),
        };
        file.write_all(DEFAULT_CONFIG_TEMPLATE.as_bytes())?;

        Ok(path)
    }
    pub(super) fn load_config_files(cli: &Cli, matches: &ArgMatches) -> Result<Self> {
        let mut config = Self::default();
        let config_paths = Self::get_config_paths(cli, matches)?;
        if let Some(first_path) = config_paths.first()
            && let Some(parent) = first_path.parent()
        {
            config.config_dir = Some(parent.to_path_buf());
        }

        if cli.no_config {
            return Ok(config);
        }

        for path in config_paths {
            if path.exists() {
                match Self::load_from_file(&path) {
                    Ok(file_config) => {
                        config.merge_with(file_config);
                        config.config_file = Some(path.clone());
                        if let Some(parent) = path.parent() {
                            config.config_dir = Some(parent.to_path_buf());
                        }
                        break;
                    }
                    Err(e) => {
                        log::warn!("Failed to load config from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(config)
    }
    fn get_config_paths(cli: &Cli, matches: &ArgMatches) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        let mut has_explicit = false;

        if let Some(config_file) = &cli.config_file
            && arg_has_user_value(matches, "config_file")
        {
            let dir = resolve_config_dir(config_file)?;
            paths.push(dir.join("config.yaml"));
            paths.push(dir.join("config.yml"));
            has_explicit = true;
        }

        if let Some(env_path) = std::env::var_os(CONFIG_FILE_ENV)
            && !env_path.is_empty()
        {
            let dir = resolve_config_dir(Path::new(&env_path))?;
            paths.push(dir.join("config.yaml"));
            paths.push(dir.join("config.yml"));
            has_explicit = true;
        }

        if !has_explicit && let Some(mdv_dir) = default_config_dir() {
            paths.push(mdv_dir.join("config.yaml"));
            paths.push(mdv_dir.join("config.yml"));
        }

        Ok(paths)
    }

    pub(super) fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;

        serde_yaml::from_str::<Self>(&content).map_err(|_| {
            anyhow::Error::from(MdvError::ConfigParseError(format!(
                "Failed to parse YAML config file: {}",
                path.display()
            )))
        })
    }
}
