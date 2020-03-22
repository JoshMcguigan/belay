use std::{
    collections::HashMap,
    convert::TryFrom,
    error::Error,
    fmt::{self, Display},
};

use yaml_rust::YamlLoader;

pub struct CiConfig {
    #[allow(dead_code)]
    pub name: String,
    pub jobs: HashMap<String, CiConfigJob>,
    pub on: Vec<CiConfigTrigger>,
}

pub struct CiConfigJob {
    pub steps: Vec<CiConfigJobStep>,
}

pub struct CiConfigJobStep {
    pub name: Option<String>,
    pub run: String,
}

pub enum CiConfigTrigger {
    Push,
    PullRequest,
}

impl TryFrom<&str> for CiConfigTrigger {
    type Error = ();

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        match input {
            "push" => Ok(CiConfigTrigger::Push),
            "pull_request" => Ok(CiConfigTrigger::PullRequest),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum YamlParseError {
    ScanError(yaml_rust::scanner::ScanError),
    MissingDocument,
    MissingField,
}

impl Display for YamlParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            YamlParseError::ScanError(e) => write!(f, "{}", e),
            YamlParseError::MissingDocument => write!(f, "No yaml document found"),
            YamlParseError::MissingField => write!(f, "Missing required field"),
        }
    }
}

impl Error for YamlParseError {}

impl From<yaml_rust::scanner::ScanError> for YamlParseError {
    fn from(input: yaml_rust::scanner::ScanError) -> Self {
        Self::ScanError(input)
    }
}

impl TryFrom<&str> for CiConfig {
    type Error = YamlParseError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let yaml = YamlLoader::load_from_str(input)?
            .pop()
            .ok_or(YamlParseError::MissingDocument)?;

        let name = &yaml["name"].as_str().ok_or(YamlParseError::MissingField)?;

        let jobs = &yaml["jobs"].as_hash().ok_or(YamlParseError::MissingField)?;

        // We attempt to parse the `on` field as both an array as well as a map,
        // then use whichever of the two worked. It would be possible to only
        // attempt to parse this as a map if parsing as an array fails, but
        // in practice there won't be any meaningful performance difference.
        let on_as_vec = yaml["on"].as_vec().map(|a| {
            a.iter()
                .filter_map(|item| {
                    item.as_str()
                        .iter()
                        .filter_map(|&s| CiConfigTrigger::try_from(s).ok())
                        .next()
                })
                .collect()
        });
        let on_as_map = yaml["on"].as_hash().map(|hashmap| {
            hashmap
                .keys()
                .filter_map(|item| item.as_str())
                .filter_map(|s| CiConfigTrigger::try_from(s).ok())
                .collect()
        });
        let on = match (on_as_vec, on_as_map) {
            (Some(x), _) => x,
            (_, Some(x)) => x,
            (None, None) => vec![],
        };

        let mut ci_config = CiConfig {
            name: (*name).to_string(),
            jobs: HashMap::new(),
            on,
        };

        for (job_name, job) in jobs.iter() {
            let job_name = job_name
                .as_str()
                .ok_or(YamlParseError::MissingField)
                .map(|s| (*s).to_string())?;

            let steps = &job["steps"].as_vec().ok_or(YamlParseError::MissingField)?;

            let mut parsed_steps = vec![];

            for step in steps.iter() {
                let name = step["name"].as_str().map(|s| (*s).to_string());
                let run = step["run"].as_str().map(|s| (*s).to_string());

                // we skip steps without run
                if let Some(run) = run {
                    let step = CiConfigJobStep { name, run };

                    parsed_steps.push(step);
                }
            }

            ci_config.jobs.insert(
                job_name,
                CiConfigJob {
                    steps: parsed_steps,
                },
            );
        }

        Ok(ci_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn parse_github_yaml() -> Result<()> {
        let github_yaml = include_str!("../../tests/github_parse_check.yml");

        let github_ci_config = CiConfig::try_from(github_yaml)?;

        assert_eq!("Rust", &github_ci_config.name);

        assert_eq!(1, github_ci_config.jobs.len());

        let job = &github_ci_config.jobs["build"];

        // the `uses` step is skipped during parsing
        assert_eq!(5, job.steps.len());

        assert_eq!(2, github_ci_config.on.len());

        Ok(())
    }

    #[test]
    fn parse_github_yaml_push_to_branch() -> Result<()> {
        let github_yaml = include_str!("../../tests/github_parse_check_on_push_to_branch.yml");

        let github_ci_config = CiConfig::try_from(github_yaml)?;

        assert_eq!(2, github_ci_config.on.len());

        Ok(())
    }
}
