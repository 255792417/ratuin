pub fn is_sensitive_command(command: &str) -> bool {
    let lower = command.to_ascii_lowercase();

    let keywords = [
        "password",
        "passwd",
        "token",
        "secret",
        "apikey",
        "api_key",
        "access_key",
        "private_key",
        "authorization",
        "bearer ",
        "cookie",
        "set-cookie",
        "aws_secret_access_key",
        "ghp_",
        "xoxb-",
    ];

    keywords.iter().any(|keyword| lower.contains(keyword))
}
