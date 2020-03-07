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

#[derive(Deserialize)]
pub struct GitlabCiConfigJob {
    script: Vec<String>,
}

impl TaskList for GitlabCiConfig {
    fn all_tasks(&self) -> Vec<Task> {
        self.jobs
            .values()
            .flat_map(|job| &job.script)
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
    fn parse_github_yaml() -> Result<()> {
        let gitlab_yaml = include_str!("../../tests/gitlab_parse_check.yml");

        let gitlab_ci_config = serde_yaml::from_str::<GitlabCiConfig>(gitlab_yaml)?;

        assert_eq!(1, gitlab_ci_config.jobs.len());

        assert_eq!(5, gitlab_ci_config.jobs["test"].script.len());

        // // all tasks returns one less than the build job, because the
        // // first step doesn't have a "run" field
        // assert_eq!(5, github_ci_config.all_tasks().len());

        // // tasks returns one less than all tasks because we
        // // don't want to `sudo apt install -y nasm`
        // assert_eq!(4, github_ci_config.tasks().len());

        Ok(())
    }
}
