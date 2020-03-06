use std::{
    env::current_dir,
    fs::{read_dir, read_to_string},
    path::PathBuf,
    process::Command,
};

mod ci;
use ci::{github::GitHubCiConfig, TaskList};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let root_dir = find_git_root().ok_or("Failed to find git root")?;

    let github_workflows_dir = {
        let mut gh = root_dir;
        gh.push(".github");
        gh.push("workflows");

        gh
    };

    let workflow = read_dir(github_workflows_dir)
        .map_err(|_| "Unable to find CI configuration")?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .next()
        .ok_or("Missing GitHub workflow")?;

    let github_ci_config = serde_yaml::from_str::<GitHubCiConfig>(&read_to_string(workflow)?)?;

    for task in github_ci_config.tasks() {
        if let Some(task_name) = &task.name {
            println!("Checking '{}':", task_name);
        } else {
            println!("Checking:");
        };
        let status = Command::new("sh").arg("-c").arg(task.command).status()?;

        if status.success() {
            println!("Success!");
        } else {
            return Err("Failed".into());
        }
    }

    Ok(())
}

fn find_git_root() -> Option<PathBuf> {
    let mut dir = current_dir().ok()?;

    loop {
        let mut git_dir = dir.clone();
        git_dir.push(".git");

        if git_dir.exists() {
            return Some(dir);
        }

        dir.push("..");

        if !dir.exists() {
            return None;
        }
    }
}
