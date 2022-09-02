use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use anyhow::Context;
use tokio::process::Command;
use crate::Config;
use crate::platform::Platform;

pub struct BuildDockerImage {
    config: Config,
    platform: Platform,
}

impl BuildDockerImage {
    pub fn new(config: Config, platform: Platform) -> BuildDockerImage {
        Self {
            config,
            platform,
        }
    }


    pub async fn run(mut self) -> Result<(), ::anyhow::Error> {

        // create tmp folder
        let tmp_binaries_folder = format!("{}/.easycontainer_tmp_binaries", &self.config.dir_project);
        ::tokio::fs::create_dir_all(&tmp_binaries_folder).await.context(format!("create directory {}", &tmp_binaries_folder))?;

        // drop old binaries
        let old_binaries = Self::find_binaries_on_path(&tmp_binaries_folder).await.context(format!("find old binaries in directory {}", &tmp_binaries_folder))?;
        for old_binary in old_binaries.iter() {
            ::tokio::fs::remove_file(old_binary).await.context(format!("could not delete old binary {}", old_binary.display()))?;
        }

        // copy new binaries
        let new_binaries = self.find_binaries().await.context("find binaries")?;
        for new_binary in new_binaries.iter() {
            let new_path = format!("{}/{}", tmp_binaries_folder, new_binary.file_name().expect("invalid filename").to_string_lossy().to_string());
            ::tokio::fs::copy(new_binary, &new_path).await.context(format!("could not copy from {} to {}", new_binary.display(), &new_path))?;
        }

        // build the docker image
        self.build_container().await.context("build docker image")?;

        Ok(())
    }

    async fn build_container(&self) -> Result<(), ::anyhow::Error> {

        let args = &[
            "build",
            "-t",
            &self.config.build_docker_platform_tag(&self.platform),
            "--build-arg","CARGO_RELEASE=.easycontainer_tmp_binaries",
            "--platform", &self.platform.docker_platform,
            &self.config.dir_project,
        ];

        println!("running docker with args: {:?}", args);

        let cmd = Command::new("docker").args(args).output().await.context("build original container")?;

        println!("{}", String::from_utf8_lossy(cmd.stdout.as_ref()));
        println!("{}", String::from_utf8_lossy(cmd.stderr.as_ref()));

        Ok(())
    }

    async fn find_binaries(&self) -> Result<Vec<PathBuf>, ::anyhow::Error> {
        let release_path = format!("{}/target/{}/release", &self.config.dir_work, &self.platform.rust_target);

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