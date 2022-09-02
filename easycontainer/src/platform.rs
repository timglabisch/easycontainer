use anyhow::{anyhow, Context};
use tokio::join;
use tokio::process::Command;
use crate::Opt;

#[derive(Debug, Clone)]
pub struct Platform {
    pub docker_platform: &'static str,
    pub rust_target: &'static str,
    pub container: String,
}

pub async fn create_platforms(config: &Opt) -> Result<Vec<Platform>, ::anyhow::Error> {

    let (amd64, arm64) = join!(
        create_platform_container(config, "linux/amd64"),
        create_platform_container(config, "linux/arm64/v8")
    );

    Ok(vec![
        Platform {
            rust_target: "x86_64-unknown-linux-musl",
            docker_platform: "linux/amd64",
            container: amd64.context("linux/amd64 platform container")?,
        },
        Platform {
            rust_target: "aarch64-unknown-linux-musl",
            docker_platform: "linux/arm64/v8",
            container: arm64.context("linux/arm64/v8 platform container")?,
        },
    ])
}

async fn create_platform_container(config: &Opt, docker_platform: &str) -> Result<String, ::anyhow::Error> {

    println!("build platform container {}", docker_platform);

    let file_or_container = config.container.clone().unwrap_or("easybill/easycontainer:latest".to_string()); // todo

    if ::tokio::fs::metadata(&file_or_container).await.context("read file").is_ok() {
        let cmd = Command::new("docker").args(&[
            "buildx",
            "build",
            "--platform", docker_platform,
            &file_or_container,
            "-q"
        ]).output().await.context("buildx")?;

        println!("finished build platform container {}", docker_platform);

        println!("{}", String::from_utf8_lossy(cmd.stdout.as_ref()));
        println!("{}", String::from_utf8_lossy(cmd.stderr.as_ref()));

        return match cmd.status.success() {
            true => Ok(String::from_utf8_lossy(cmd.stdout.as_ref()).trim().to_string()),
            false => Err(anyhow!("could not build container"))
        };
    }


    panic!("invalid file or container {}", &file_or_container);
}