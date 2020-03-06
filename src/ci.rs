pub mod github;

pub struct Task {
    pub name: Option<String>,
    pub command: String,
}

pub trait TaskList {
    /// Returns all CI tasks, including tasks which we
    /// would not want to execute in belay.
    fn all_tasks(&self) -> Vec<Task>;

    /// Returns the subset of CI tasks that we do
    /// want to execute in belay.
    fn tasks(&self) -> Vec<Task> {
        self.all_tasks()
            .into_iter()
            .filter(|task| {
                let command_blacklist = vec!["apt install"];

                for blacklisted_command in command_blacklist {
                    if task.command.contains(blacklisted_command) {
                        return false;
                    }
                }

                true
            })
            .collect()
    }
}
