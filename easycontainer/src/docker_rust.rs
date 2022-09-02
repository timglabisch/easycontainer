use std::path::PathBuf;
use anyhow::{anyhow, Context};
use tokio::process::Command;
use crate::Config;
use crate::platform::Platform;

pub struct ProjectDockerRust {

}

impl ProjectDockerRust {
    pub async fn new(config: Config, platforms: Vec<Platform>) -> Result<Self, ::anyhow::Error> {
        let dockerfile = format!("{}/Dockerfile", &config.dir_project);
        if ::tokio::fs::metadata(&dockerfile).await.is_err() {
            return Err(anyhow!(format!("Dockerfile does not exists, file: {}", &dockerfile)));
        }

        if ::tokio::fs::metadata(format!("{}/Cargo.toml", &config.dir_project)).await.is_err() {
            return Err(anyhow!("Cargo.toml does not exists"));
        }

        println!("Detect Docker Rust Project");

        for platform in platforms.iter() {
            Self::build(&config, &platform).await.context("build platform")?;
        }

        Ok(Self {})
    }

    pub async fn build(config : &Config, platform : &Platform) -> Result<(), ::anyhow::Error> {
        let args = &[
            "run",  "--rm",
            "--entrypoint", "cargo",
            "--workdir", &config.dir_project,
            "--platform", platform.docker_platform,
            "-e",  "CARGO_HOME=/cargo",
            "--network", "host",
            "-v", &format!("{}:{}", &config.dir_work, &config.dir_work),
            "-v", "/tmp/.cargo_cache:/cargo",
            &platform.container,
            "build",
            "--release",
            "--target", platform.rust_target
        ];

        println!("running docker wirth args: {:?}", args);

        let cmd = Command::new("docker").args(args).output().await.context("build")?;

        println!("{}", String::from_utf8_lossy(cmd.stdout.as_ref()));
        println!("{}", String::from_utf8_lossy(cmd.stderr.as_ref()));


        Ok(())
    }
}