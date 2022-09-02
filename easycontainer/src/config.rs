use crate::platform::Platform;

#[derive(Debug, Clone)]
pub struct Config {
    pub dir_work: String,
    pub dir_project: String,
    pub docker_tag: String,
}

impl Config {
    pub fn build_docker_platform_tag(&self, platform: &Platform) -> String {
        let tag = platform.docker_platform
            .replace("linux", "")
            .replace("/", "_")
            .replace("__", "_");

        format!("{}_{}", &self.docker_tag, tag.trim_matches('_'))
    }
}