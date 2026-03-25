use anyhow::Result;

use crate::ratuin::cli::{Cli, Commands};

mod handlers;
mod validators;

pub fn run(cli: Cli) -> Result<()> {
    validators::validate_command(&cli.command)?;

    let conn = crate::ratuin::db::open_db()?;

    match cli.command {
        Commands::Init { shell } => handlers::handle_init(shell),
        Commands::Import {
            shell,
            file,
            allow_sensitive,
        } => handlers::handle_import(&conn, shell, file, allow_sensitive),
        Commands::Record {
            command,
            cwd,
            exit_code,
            duration_ms,
            allow_sensitive,
        } => handlers::handle_record(&conn, command, cwd, exit_code, duration_ms, allow_sensitive),
        Commands::Search {
            keyword,
            limit,
            cwd,
            failed_only,
        } => handlers::handle_search(&conn, keyword, limit, cwd, failed_only),
        Commands::Tui {
            keyword,
            cwd,
            limit,
            failed_only,
            forward,
        } => handlers::handle_tui(keyword, cwd, limit, failed_only, forward),
    }
}
