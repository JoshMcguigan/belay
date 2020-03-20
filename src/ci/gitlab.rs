use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct CiConfig {
    #[allow(dead_code)]
    image: Option<String>,
    #[allow(dead_code)]
    stages: Option<Vec<String>>,
    #[serde(flatten)]
    pub(super) jobs: HashMap<String, CiConfigJob>,
}

/// All fields which aren't explicitly configured in this struct are
/// parsed as `jobs`, since jobs can have (almost) any name.
///
/// Although all actual jobs will have a script field, the field
/// is marked as optional here to support parsing config files
/// which have extra fields in the root (see the `cache` key
/// in the example gitlab file).
#[derive(Deserialize)]
pub struct CiConfigJob {
    pub(super) script: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn parse_gitlab_yaml() -> Result<()> {
        let gitlab_yaml = include_str!("../../tests/gitlab_parse_check.yml");

        let gitlab_ci_config = serde_yaml::from_str::<CiConfig>(gitlab_yaml)?;

        assert_eq!(
            5,
            gitlab_ci_config.jobs.values().fold(0, |mut acc, job| {
                acc += job
                    .script
                    .as_ref()
                    .map(|scripts| scripts.len())
                    .unwrap_or(0);
                acc
            })
        );

        Ok(())
    }
}
