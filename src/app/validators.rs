use anyhow::Result;

use crate::cli::Commands;

const SUPPORTED_SHELLS: [&str; 3] = ["bash", "zsh", "fish"];

fn validate_shell(shell: &str) -> Result<()> {
    if !SUPPORTED_SHELLS.contains(&shell) {
        anyhow::bail!(
            "Unsupported shell: {}. Supported shells are: {}",
            shell,
            SUPPORTED_SHELLS.join(", ")
        );
    }

    Ok(())
}

fn validate_record(command: &str) -> Result<()> {
    if command.trim().is_empty() {
        anyhow::bail!("Command cannot be empty");
    }

    Ok(())
}

fn validate_search(limit: Option<usize>, cwd: Option<&str>) -> Result<()> {
    if let Some(limit) = limit
        && limit == 0
    {
        anyhow::bail!("Search limit must be greater than 0");
    }

    if let Some(cwd) = cwd
        && cwd.trim().is_empty()
    {
        anyhow::bail!("Search cwd cannot be empty");
    }

    Ok(())
}

fn validate_import(shell: &str, file: Option<&std::path::Path>) -> Result<()> {
    validate_shell(shell)?;

    if let Some(path) = file
        && !path.exists()
    {
        anyhow::bail!("History file does not exist: {}", path.display());
    }

    Ok(())
}

pub fn validate_command(command: &Commands) -> Result<()> {
    match command {
        Commands::Record { command, .. } => validate_record(command),
        Commands::Search { limit, cwd, .. } => validate_search(*limit, cwd.as_deref()),
        Commands::Tui { limit, cwd, .. } => validate_search(*limit, cwd.as_deref()),
        Commands::Import { shell, file, .. } => validate_import(shell, file.as_deref()),
        Commands::Init { shell } => validate_shell(shell),
    }
}
