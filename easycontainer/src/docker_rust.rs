use std::path::PathBuf;
use anyhow::{anyhow, Context};
use tokio::process::Command;
use crate::platform::Platform;

pub struct ProjectDockerRust {

}

impl ProjectDockerRust {
    pub async fn new(path : &PathBuf, platforms: Vec<Platform>) -> Result<Self, ::anyhow::Error> {
        let dockerfile = format!("{}/Dockerfile", path.display());
        if ::tokio::fs::metadata(&dockerfile).await.is_err() {
            return Err(anyhow!(format!("Dockerfile does not exists, file: {}", &dockerfile)));
        }

        if ::tokio::fs::metadata(format!("{}/Cargo.toml", path.display())).await.is_err() {
            return Err(anyhow!("Cargo.toml does not exists"));
        }

        println!("Detect Docker Rust Project");

        for platform in platforms.iter() {
            Self::build(path, &platform).await.context("build platform")?;
        }

        Ok(Self {})
    }

    pub async fn build(path : &PathBuf, platform : &Platform) -> Result<(), ::anyhow::Error> {
        let cmd = Command::new("docker").args(&[
            "run",  "--rm",
            "--entrypoint=cargo",
            &format!("--workdir={}", path.display()),
            "--platform", platform.docker_platform,
            "-e",  "CARGO_HOME=\"/cargo\"",
            "--network", "host",
            &format!("-v \"{}:{}\"", path.display(), path.display()),
            "-v", "/tmp/.cargo_cache:/cargo",
            &platform.container,
            "build",
            "--release",
            "--target", platform.rust_target
        ]);

        Ok(())
    }
}