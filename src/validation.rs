use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Alias name cannot be empty")]
    EmptyName,

    #[error("Invalid alias name syntax: {0}")]
    InvalidSyntax(String),

    #[error("Protected command name '{0}' requires --force to shadow")]
    ProtectedName(String),
}

/// Validate that an alias name matches the required syntax: [A-Za-z_][A-Za-z0-9_-]*
pub fn validate_alias_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        return Err(ValidationError::EmptyName);
    }

    let mut chars = name.chars();
    let first = chars.next();

    // First character must be [A-Za-z_]
    match first {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return Err(ValidationError::InvalidSyntax(name.to_string())),
    }

    // Remaining characters must be [A-Za-z0-9_-]
    for c in chars {
        if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
            continue;
        }
        return Err(ValidationError::InvalidSyntax(name.to_string()));
    }

    Ok(())
}

/// Check if a name is in the protected set of shell commands.
pub fn is_protected_name(name: &str) -> bool {
    matches!(
        name,
        "rm"
            | "mv"
            | "cp"
            | "ln"
            | "chmod"
            | "chown"
            | "kill"
            | "sudo"
            | "su"
            | "cd"
            | "source"
            | "exec"
            | "eval"
            | "export"
            | "unset"
            | "exit"
            | "logout"
            | "git"
            | "ssh"
            | "curl"
            | "wget"
            | "brew"
    )
}

/// Validate an alias name for write. Protected names require `force = true`.
pub fn validate_alias_name_for_write(name: &str, force: bool) -> Result<(), ValidationError> {
    validate_alias_name(name)?;

    if is_protected_name(name) && !force {
        return Err(ValidationError::ProtectedName(name.to_string()));
    }

    Ok(())
}
