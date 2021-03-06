use std::{
    collections::HashSet,
    convert::TryFrom,
    env::current_dir,
    fs::{read_dir, read_to_string, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
use structopt::StructOpt;

#[cfg(not(windows))]
use std::{fs::Permissions, os::unix::fs::PermissionsExt};

mod args;
use args::{Args, Subcommand};

mod ci;
use ci::{github, gitlab, Task, TaskList, Trigger};

mod config;
use config::Config;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let root_dir = find_git_root().ok_or("Failed to find git root")?;

    let args = Args::from_args();

    if let Some(Subcommand::Hook { hook_type }) = args.subcommand {
        let hook_filename = hook_type.filename();
        let mut hook_path = root_dir;
        hook_path.push(".git");
        hook_path.push("hooks");
        hook_path.push(&hook_filename);

        let mut file = File::create(&hook_path)?;

        #[cfg(not(windows))]
        file.set_permissions(Permissions::from_mode(0o755))?;

        file.write_all(b"#!/bin/sh\nbelay")?;

        println!("Created hook `.git/hooks/{}`", hook_filename);

        return Ok(());
    }

    let ci_configs: Vec<Box<dyn TaskList>> =
        match (handle_github(&root_dir), handle_gitlab(&root_dir)) {
            (Ok(configs), _) => configs
                .into_iter()
                .map(|c| Box::new(c) as Box<dyn TaskList>)
                .collect(),
            (_, Ok(config)) => vec![Box::new(config)],
            _ => return Err("Unable to find CI configuration".into()),
        };

    let mut completed_commands = HashSet::new();
    for ci_config in ci_configs {
        for task in ci_config.tasks(Config::read(), get_triggers()) {
            let Task { name, command, .. } = task;

            // we want to de-duplicate commands across CI configurations
            if completed_commands.contains(&command) {
                continue;
            }

            let task_name = name.unwrap_or_else(|| command.clone());
            println!("Checking '{}':", task_name);

            #[cfg(not(windows))]
            let status = Command::new("sh").arg("-c").arg(&command).status()?;
            #[cfg(windows)]
            let status = Command::new("cmd").arg("/c").arg(&command).status()?;

            if status.success() {
                println!("Success!");
            } else {
                return Err("Failed".into());
            }
            completed_commands.insert(command);
        }
    }

    Ok(())
}

fn handle_github(root_dir: &Path) -> Result<Vec<github::CiConfig>> {
    let github_workflows_dir = {
        let mut gh = root_dir.to_path_buf();
        gh.push(".github");
        gh.push("workflows");

        gh
    };

    let mut paths = read_dir(github_workflows_dir)
        .map_err(|_| "Unable to find CI configuration")?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<PathBuf>>();
    // Sort the workflow files alphabetically, so they run
    // in deterministic order.
    paths.sort();

    let configs = paths
        .into_iter()
        .map(|path| -> Result<github::CiConfig> {
            Ok(github::CiConfig::try_from(read_to_string(path)?.as_str())?)
        })
        .collect::<Result<Vec<github::CiConfig>>>()?;

    if configs.is_empty() {
        return Err("failed to find github config".into());
    }

    Ok(configs)
}

fn handle_gitlab(root_dir: &Path) -> Result<gitlab::CiConfig> {
    let file_path = {
        let mut path = root_dir.to_path_buf();
        path.push(".gitlab-ci.yml");

        path
    };

    Ok(serde_yaml::from_str::<gitlab::CiConfig>(&read_to_string(
        file_path,
    )?)?)
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

/// Get the best estimate of the triggers for this CI run.
///
/// We can't know for sure if this will turn into a pull
/// request, but we assume if there is an upstream remote
/// that it will.
fn get_triggers() -> Vec<Trigger> {
    let mut triggers = vec![Trigger::Push {
        branch: current_branch(),
    }];

    if has_upstream() {
        triggers.push(Trigger::PullRequest);
    }

    triggers
}

/// Used to guess if this will turn into a pull request. Has the
/// limitation that it only works if the upstream repository is
/// named 'upstream'.
fn has_upstream() -> bool {
    let command = "git remote";
    #[cfg(not(windows))]
    let output = Command::new("sh").arg("-c").arg(&command).output();
    #[cfg(windows)]
    let output = Command::new("cmd").arg("/c").arg(&command).output();

    let output = output.expect("failed to run git command");

    assert!(output.status.success(), "command to get git remotes failed");

    let remotes = String::from_utf8_lossy(&output.stdout).into_owned();
    remotes.contains("upstream")
}

fn current_branch() -> String {
    let command = "git rev-parse --abbrev-ref HEAD";
    #[cfg(not(windows))]
    let output = Command::new("sh").arg("-c").arg(&command).output();
    #[cfg(windows)]
    let output = Command::new("cmd").arg("/c").arg(&command).output();

    let output = output.expect("failed to run git command");

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout).into_owned()
    } else {
        // We assume that if this command fails it is because no commits exist in this
        // repository. In that case, use 'master' as a placeholder branch name. This is
        // unlikely to happen often in real-world usage, but it happens in integration tests.
        "master".into()
    }
}
