use serde::Deserialize;
use std::collections::HashMap;

use super::{Task, TaskList};

#[derive(Deserialize)]
pub struct GitHubCiConfig {
    #[allow(dead_code)]
    name: String,
    jobs: HashMap<String, GitHubCiConfigJob>,
}

#[derive(Deserialize)]
pub struct GitHubCiConfigJob {
    steps: Vec<GitHubCiConfigJobStep>,
}

#[derive(Deserialize)]
pub struct GitHubCiConfigJobStep {
    name: Option<String>,
    run: Option<String>,
}

impl TaskList for GitHubCiConfig {
    fn all_tasks(&self) -> Vec<Task> {
        self.jobs
            .values()
            .flat_map(|job| &job.steps)
            .filter_map(|step| {
                if let Some(command) = &step.run {
                    Some(Task {
                        name: step.name.clone(),
                        command: command.clone(),
                    })
                } else {
                    None
                }
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
        let github_yaml = include_str!("../../tests/github_parse_check.yml");

        let github_ci_config = serde_yaml::from_str::<GitHubCiConfig>(github_yaml)?;

        assert_eq!("Rust", &github_ci_config.name);
        assert_eq!(1, github_ci_config.jobs.len());

        assert_eq!(6, github_ci_config.jobs["build"].steps.len());

        // all tasks returns one less than the build job, because the
        // first step doesn't have a "run" field
        assert_eq!(5, github_ci_config.all_tasks().len());

        // tasks returns one less than all tasks because we
        // don't want to `sudo apt install -y nasm`
        assert_eq!(4, github_ci_config.tasks().len());

        Ok(())
    }
}
