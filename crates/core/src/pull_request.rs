use std::{fmt, str::FromStr};

use serde::Serialize;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PullRequestRef {
    pub owner: String,
    pub repository: String,
    pub number: u64,
}

impl PullRequestRef {
    pub fn github_url(&self) -> String {
        format!(
            "https://github.com/{}/{}/pull/{}",
            self.owner, self.repository, self.number
        )
    }
}

impl fmt::Display for PullRequestRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}/{}#{}",
            self.owner, self.repository, self.number
        )
    }
}

impl FromStr for PullRequestRef {
    type Err = PullRequestRefError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.trim() != value {
            return Err(PullRequestRefError::InvalidFormat);
        }

        let (slug, number) = value
            .split_once('#')
            .ok_or(PullRequestRefError::InvalidFormat)?;
        if number.contains('#') {
            return Err(PullRequestRefError::InvalidFormat);
        }

        let (owner, repository) = slug
            .split_once('/')
            .ok_or(PullRequestRefError::InvalidFormat)?;
        if repository.contains('/') || !valid_owner(owner) || !valid_repository(repository) {
            return Err(PullRequestRefError::InvalidFormat);
        }

        let number = number
            .parse::<u64>()
            .map_err(|_| PullRequestRefError::InvalidNumber)?;
        if number == 0 {
            return Err(PullRequestRefError::InvalidNumber);
        }

        Ok(Self {
            owner: owner.to_owned(),
            repository: repository.to_owned(),
            number,
        })
    }
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn valid_repository(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

#[derive(Debug, Error)]
pub enum PullRequestRefError {
    #[error("expected OWNER/REPO#NUMBER (for example github/desktop#144)")]
    InvalidFormat,
    #[error("pull request number must be a positive integer")]
    InvalidNumber,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::PullRequestRef;

    #[test]
    fn parses_pull_request_reference() {
        let pull_request =
            PullRequestRef::from_str("github/desktop#144").expect("valid pull request reference");

        assert_eq!(pull_request.owner, "github");
        assert_eq!(pull_request.repository, "desktop");
        assert_eq!(pull_request.number, 144);
        assert_eq!(
            pull_request.github_url(),
            "https://github.com/github/desktop/pull/144"
        );
    }

    #[test]
    fn rejects_invalid_pull_request_references() {
        for value in [
            "github/desktop",
            "github/desktop#0",
            "github/desktop#abc",
            "github/desktop/pull#144",
            "-github/desktop#144",
            "github/desktop #144",
        ] {
            assert!(
                PullRequestRef::from_str(value).is_err(),
                "{value} should be rejected"
            );
        }
    }
}
