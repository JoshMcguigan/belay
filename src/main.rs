use std::{
    env::current_dir,
    fs::{read_dir, read_to_string, File, Permissions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};
use structopt::StructOpt;

mod ci;
use ci::{github::GitHubCiConfig, gitlab::GitlabCiConfig, TaskList};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(StructOpt)]
struct Args {
    #[structopt(subcommand)]
    subcommand: Option<Subcommand>,
}

#[derive(StructOpt)]
enum Subcommand {
    Hook {
        #[structopt(subcommand)]
        hook_type: HookType,
    },
}

#[derive(StructOpt)]
enum HookType {
    Commit,
    Push,
}

impl HookType {
    fn filename(&self) -> String {
        match self {
            HookType::Commit => String::from("pre-commit"),
            HookType::Push => String::from("pre-push"),
        }
    }
}

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
        file.set_permissions(Permissions::from_mode(0o755))?;

        file.write_all(b"#!/usr/bin/sh\nbelay")?;

        println!("Created hook `.git/hooks/{}`", hook_filename);

        return Ok(());
    }

    let ci_config: Box<dyn TaskList> = match (handle_github(&root_dir), handle_gitlab(&root_dir)) {
        (Ok(config), _) => Box::new(config),
        (_, Ok(config)) => Box::new(config),
        _ => return Err("Unable to find CI configuration".into()),
    };

    for task in ci_config.tasks() {
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

fn handle_github(root_dir: &Path) -> Result<GitHubCiConfig> {
    let github_workflows_dir = {
        let mut gh = root_dir.to_path_buf();
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

    Ok(serde_yaml::from_str::<GitHubCiConfig>(&read_to_string(
        workflow,
    )?)?)
}

fn handle_gitlab(root_dir: &Path) -> Result<GitlabCiConfig> {
    let file_path = {
        let mut path = root_dir.to_path_buf();
        path.push(".gitlab-ci.yml");

        path
    };

    Ok(serde_yaml::from_str::<GitlabCiConfig>(&read_to_string(
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
