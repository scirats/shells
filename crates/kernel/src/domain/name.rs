use crate::error::DomainError;

const MAX_LENGTH: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShellName(String);

impl ShellName {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        if is_valid(raw) {
            Ok(Self(raw.to_string()))
        } else {
            Err(DomainError::InvalidName(raw.to_string()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ShellName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn is_valid(raw: &str) -> bool {
    !raw.is_empty() && raw.len() <= MAX_LENGTH && has_valid_body(raw) && has_valid_tail(raw)
}

fn has_valid_body(raw: &str) -> bool {
    raw.chars()
        .enumerate()
        .all(|(index, c)| c.is_ascii_lowercase() || c.is_ascii_digit() || (c == '-' && index > 0))
}

fn has_valid_tail(raw: &str) -> bool {
    raw.chars().last().is_some_and(|c| c != '-')
}
