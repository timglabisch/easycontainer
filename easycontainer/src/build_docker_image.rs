use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use anyhow::Context;
use tokio::process::Command;
use crate::{Config, platform};
use crate::platform::Platform;

pub struct BuildDockerImage {
    config: Config,
    platforms: Vec<Platform>,
}

impl BuildDockerImage {
    pub fn new(config: Config, platforms: Vec<Platform>) -> BuildDockerImage {
        Self {
            config,
            platforms: platforms,
        }
    }

    pub async fn tidy_up(binary_folder : &str) -> Result<(), ::anyhow::Error> {
        // drop old binaries
        let old_binaries = Self::find_binaries_on_path(binary_folder).await.context(format!("find old binaries in directory {}", &binary_folder))?;
        for old_binary in old_binaries.iter() {
            ::tokio::fs::remove_file(old_binary).await.context(format!("could not delete old binary {}", old_binary.display()))?;
        }

        Ok(())
    }

    pub async fn run(self) -> Result<(), ::anyhow::Error> {

        let mut jhs = vec![];
        for platform in &self.platforms {
            let spawn_platform = platform.clone();
            let spawn_config = self.config.clone();
            jhs.push(::tokio::spawn(async move {
                Self::run_for_platform(spawn_config, spawn_platform).await;
                ()
            }));
        }

        ::futures::future::join_all(jhs).await;

        // create the manifest
        // self.create_manifest().await?;

        Ok(())
    }

    pub async fn run_for_platform(config: Config, platform: Platform) -> Result<(), ::anyhow::Error> {

        println!("building container for platform {}", &platform.docker_platform);

        // create tmp folder
        let tmp_binaries_folder = format!("{}/.eb_build_tmp_{}", &config.dir_project, Config::build_docker_platform_tag_arch(&platform));
        ::tokio::fs::create_dir_all(&tmp_binaries_folder).await.context(format!("create directory {}", &tmp_binaries_folder))?;

        Self::tidy_up(&tmp_binaries_folder).await?;


        // copy new binaries
        let new_binaries = Self::find_binaries(&config, &platform).await.context("find binaries")?;
        for new_binary in new_binaries.iter() {
            let new_path = format!("{}/{}", &tmp_binaries_folder, new_binary.file_name().expect("invalid filename").to_string_lossy().to_string());
            println!("copy binary from {} to {}", new_binary.display(), &new_path);
            ::tokio::fs::copy(new_binary, &new_path).await.context(format!("could not copy from {} to {}", new_binary.display(), &new_path))?;
        }

        // build the docker image
        Self::build_container(&config, &platform, &tmp_binaries_folder).await.context("build docker image")?;
        // Self::tidy_up(&tmp_binaries_folder).await?;

        // ::tokio::fs::remove_dir(&tmp_binaries_folder).await.context(format!("delete directory {}", &tmp_binaries_folder))?;

        println!("finished building container for platform {}", &platform.docker_platform);

        Ok(())
    }

    async fn create_manifest(&self) -> Result<(), ::anyhow::Error> {

        unimplemented!("we dont create the manifest yet, you need to have access to the registry to work with the manifest");

        let mut platform_tags = self.platforms.iter().map(|p| self.config.build_docker_platform_tag(p)).collect::<Vec<_>>();

        let mut args = [
            "manifest",
            "create",
            &self.config.docker_tag,
        ].iter().map(|x|x.to_string()).collect::<Vec<_>>();

        args.append(&mut platform_tags);

        println!("running docker with args: {:?}", args);

        let cmd = Command::new("docker").args(args).output().await.context("build original container")?;

        println!("{}", String::from_utf8_lossy(cmd.stdout.as_ref()));
        println!("{}", String::from_utf8_lossy(cmd.stderr.as_ref()));

        Ok(())
    }

    async fn build_container(config: &Config, platform: &Platform, binary_folder: &str) -> Result<(), ::anyhow::Error> {

        let relative_tmp_binaries_folder = format!(
            "./.eb_build_tmp_{}",
            Config::build_docker_platform_tag_arch(&platform)
        );

        let args = &[
            "build",
            "-t",
            &config.build_docker_platform_tag(&platform),
            "--build-arg", &format!("CARGO_RELEASE={}", relative_tmp_binaries_folder),
            "--platform", &platform.docker_platform,
            &config.dir_project,
        ];

        println!("running docker with args: {:?}", args);

        let cmd = Command::new("docker").args(args).output().await.context("build original container")?;

        println!("{}", String::from_utf8_lossy(cmd.stdout.as_ref()));
        println!("{}", String::from_utf8_lossy(cmd.stderr.as_ref()));

        Ok(())
    }

    async fn find_binaries(config: &Config, platform: &Platform) -> Result<Vec<PathBuf>, ::anyhow::Error> {
        let release_path = format!("{}/target/{}/release", &config.dir_work, &platform.rust_target);

        Self::find_binaries_on_path(&release_path).await
    }

    async fn find_binaries_on_path(path : &str) -> Result<Vec<PathBuf>, ::anyhow::Error> {
        let mut binaries_search_path = ::tokio::fs::read_dir(&path).await.context(format!("read dir {}", &path))?;

        let mut binaries = vec![];

        loop {
            let binary = match binaries_search_path.next_entry().await.context("read next entry")? {
                None => break,
                Some(s) => s,
            };

            let metadata = binary.metadata().await.context("could not fetch file metadata")?;

            if !metadata.is_file() {
                continue;
            }

            // executable?
            if metadata.permissions().mode() & 0o111 == 0 {
                continue;
            }

            binaries.push(binary.path());
        }

        Ok(binaries)
    }
}