use std::path::PathBuf;
use anyhow::{anyhow, Context};
use tracing::{debug, info};
use tracing_subscriber;
use structopt::StructOpt;
use crate::docker_rust::ProjectDockerRust;
use crate::platform::create_platforms;

mod docker_rust;
mod platform;

#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "easycontainer", about = "Easycontainer Cli")]
pub struct Opt {
    #[structopt(parse(from_os_str))]
    pub input: PathBuf,

    #[structopt(long = "container")]
    pub container: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), ::anyhow::Error> {
    tracing_subscriber::fmt::init();
    let opt : Opt = Opt::from_args();

    println!("EasyContainer");

    let platforms = create_platforms(&opt).await.context("create platform")?;

    let project = ProjectDockerRust::new(&opt.input, platforms).await?;




    Ok(())
}