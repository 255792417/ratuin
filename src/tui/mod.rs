use anyhow::Result;

#[derive(Debug, Clone)]
pub struct TuiRequest {
    pub keyword: Option<String>,
    pub cwd: Option<String>,
    pub limit: Option<usize>,
    pub failed_only: bool,
}

pub fn run(request: TuiRequest) -> Result<()> {
    let initial_keyword = request.keyword.unwrap_or_default();

    println!(
        "TUI mode is not implemented yet. Parsed request => keyword: '{}', cwd: {:?}, limit: {:?}, failed_only: {}",
        initial_keyword, request.cwd, request.limit, request.failed_only
    );

    Ok(())
}
