use serde::Deserialize;
use std::collections::HashMap;

use super::{Task, TaskList};

#[derive(Deserialize)]
pub struct GitlabCiConfig {
    #[allow(dead_code)]
    image: Option<String>,
    #[allow(dead_code)]
    stages: Option<Vec<String>>,
    #[serde(flatten)]
    jobs: HashMap<String, GitlabCiConfigJob>,
}

/// All fields which aren't explicitly configured in this struct are
/// parsed as `jobs`, since jobs can have (almost) any name.
///
/// Although all actual jobs will have a script field, the field
/// is marked as optional here to support parsing config files
/// which have extra fields in the root (see the `cache` key
/// in the example gitlab file).
#[derive(Deserialize)]
pub struct GitlabCiConfigJob {
    script: Option<Vec<String>>,
}

impl TaskList for GitlabCiConfig {
    fn all_tasks(&self) -> Vec<Task> {
        self.jobs
            .values()
            .filter_map(|job| job.script.as_ref())
            .flat_map(|script: &Vec<String>| script)
            .map(|cmd| Task {
                name: None,
                command: cmd.clone(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn parse_gitlab_yaml() -> Result<()> {
        let gitlab_yaml = include_str!("../../tests/gitlab_parse_check.yml");

        let gitlab_ci_config = serde_yaml::from_str::<GitlabCiConfig>(gitlab_yaml)?;

        // tasks returns two less than all tasks because we
        // don't want to `rustup component add`
        assert_eq!(3, gitlab_ci_config.tasks().len());

        Ok(())
    }
}
