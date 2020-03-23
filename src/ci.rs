pub mod github;
pub mod gitlab;

pub struct Task {
    pub name: Option<String>,
    pub command: String,
    applicability: Vec<Applicability>,
}

/// Applicability represents the times when a task should be run.
///
/// See Trigger for additional details.
#[derive(Clone)]
pub enum Applicability {
    /// This task should only be run on a push. If branches are
    /// specified it should only be run on a push to these particular
    /// branches.
    ///
    /// Belay will run these tasks even on pre-commit hooks (as well
    /// as pre-push hooks).
    Push { branches: Option<Vec<String>> },
    /// This tasks should be run on pull requests.
    PullRequest,
    /// This task is always applicable.
    Any,
}

/// Trigger represents the type of CI event we expect to happen.
///
/// Of course we can't know whether the user has an open pull
/// request (or will open a pull request), but we can assume this
/// based on whether they have an upstream remote configured.
pub enum Trigger {
    Push { branch: String },
    PullRequest,
}

impl Applicability {
    fn is_triggered_by(&self, trigger: &Trigger) -> bool {
        match (self, trigger) {
            (
                Applicability::Push {
                    branches: Some(branches),
                },
                Trigger::Push { branch },
            ) => branches.contains(branch),
            (Applicability::Push { branches: None }, Trigger::Push { .. }) => true,
            (Applicability::PullRequest, Trigger::PullRequest) => true,
            (Applicability::Any, _) => true,
            (_, _) => false,
        }
    }
}

pub trait TaskList {
    /// Returns all CI tasks, including tasks which we
    /// would not want to execute in belay.
    fn all_tasks(&self) -> Vec<Task>;

    /// Returns the subset of CI tasks that we do
    /// want to execute in belay.
    fn tasks(&self, triggers: Vec<Trigger>) -> Vec<Task> {
        fn is_applicable(applicabilities: &[Applicability], triggers: &[Trigger]) -> bool {
            for applicability in applicabilities {
                for trigger in triggers {
                    if applicability.is_triggered_by(trigger) {
                        return true;
                    }
                }
            }

            false
        }

        self.all_tasks()
            .into_iter()
            .filter(|task| {
                let command_blacklist = vec!["apt install", "rustup component add"];

                for blacklisted_command in command_blacklist {
                    if task.command.contains(blacklisted_command) {
                        return false;
                    }
                }

                true
            })
            .filter(|task| is_applicable(&task.applicability, &triggers))
            .collect()
    }
}

impl TaskList for github::CiConfig {
    fn all_tasks(&self) -> Vec<Task> {
        self.jobs
            .values()
            .flat_map(|job| &job.steps)
            .map(|step| Task {
                name: step.name.clone(),
                command: step.run.clone(),
                applicability: self.on.clone(),
            })
            .collect()
    }
}

impl TaskList for gitlab::CiConfig {
    fn all_tasks(&self) -> Vec<Task> {
        self.jobs
            .values()
            .filter_map(|job| job.script.as_ref())
            .flat_map(|script: &Vec<String>| script)
            .map(|cmd| Task {
                name: None,
                command: cmd.clone(),
                // For now restricted applicability is not supported
                // for gitlab within belay.
                applicability: vec![Applicability::Any],
            })
            .collect()
    }
}
