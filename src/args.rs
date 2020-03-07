use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Args {
    #[structopt(subcommand)]
    pub subcommand: Option<Subcommand>,
}

#[derive(StructOpt)]
pub enum Subcommand {
    Hook {
        #[structopt(subcommand)]
        hook_type: HookType,
    },
}

#[derive(StructOpt)]
pub enum HookType {
    Commit,
    Push,
}

impl HookType {
    pub fn filename(&self) -> String {
        match self {
            HookType::Commit => String::from("pre-commit"),
            HookType::Push => String::from("pre-push"),
        }
    }
}
