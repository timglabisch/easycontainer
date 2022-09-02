use std::path::PathBuf;
use anyhow::{anyhow, Context};
use tracing::{debug, info};
use tracing_subscriber;
use structopt::StructOpt;
use crate::build_docker_image::BuildDockerImage;
use crate::config::Config;
use crate::docker_rust::ProjectDockerRust;
use crate::platform::create_platforms;

mod docker_rust;
mod platform;
mod config;
mod build_docker_image;

#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "easycontainer", about = "Easycontainer Cli")]
pub struct Opt {
    #[structopt(parse(from_os_str))]
    pub input: PathBuf,

    #[structopt(long = "container")]
    pub container: Option<String>,

    #[structopt(long = "docker-tag")]
    pub docker_tag: String,
}

#[tokio::main]
async fn main() -> Result<(), ::anyhow::Error> {
    tracing_subscriber::fmt::init();
    let opt : Opt = Opt::from_args();

    println!("EasyContainer");

    let config = Config {
        dir_project: format!("{}/{}", ::std::env::current_dir().context("current_dir")?.display(), &opt.input.display()),
        dir_work: ::std::env::current_dir().context("current_dir")?.display().to_string(),
        docker_tag: opt.docker_tag.to_string(),
    };

    // rust runtime for every platform
    let platforms = create_platforms(&opt).await.context("create platform")?;

    // compile binaries for every platform
    let project = ProjectDockerRust::new(config.clone(), platforms.clone()).await?;

    // build the original docker containers.
    BuildDockerImage::new(config.clone(), platforms.clone()).run().await.context("build docker image")?;


    println!("done!");

    Ok(())
}
