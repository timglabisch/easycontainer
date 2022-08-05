use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;
use anyhow::{anyhow, Context, Error};
use futures::{join, try_join};
use futures::future::try_join_all;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::process::Command;
use tokio::task::{JoinError, JoinHandle};
use tracing::field::debug;
use crate::process::Service;
use tracing::{debug, info};

pub async fn create_services_from_glob(glob_pattern : &str) -> Result<Vec<Service>, ::anyhow::Error> {
    let mut services = vec![];
    for entry in ::glob::glob(glob_pattern).context("Failed to read glob pattern")? {
        let path = &entry?;
        debug!("found service {}", &path.to_string_lossy());
        services.push(create_service_from_file(&path).await?);
    }

    debug!("found {} services.", services.len());

    Ok(services)
}

pub async fn run_services_from_glob(glob_pattern : &str) -> Result<Vec<()>, ::anyhow::Error> {
    let services = create_services_from_glob(glob_pattern).await?;
    let service_runs = services.iter().map(|s| Box::pin(run_service(s))).collect::<Vec<_>>();

    info!("starting {} services", service_runs.len());
    let (result, _, _) = ::futures::future::select_all(service_runs).await;
    info!("one service finished. so we will start to tidy up.");

    Err(anyhow!("panic"))
}

pub async fn create_service_from_file(path: &PathBuf) -> Result<Service, ::anyhow::Error> {
    let mut file = tokio::fs::File::open(path)
        .await
        .context(format!("could not open path {}", path.display()))
        ?;

    let mut buf = vec![];
    file.read_to_end(&mut buf).await.context(format!("could not read file {}", path.display()))?;

    let service = toml::from_slice::<Service>(&buf)
        .context(format!("could not parse toml {}", path.display()))?;

    Ok(service)
}

pub async fn run_service(service: &Service) -> Result<(), ::anyhow::Error> {
    let mut cmd = Command::new(service.command.clone());
    let cmd = cmd
        .args(&service.args)
        .stdin(::std::process::Stdio::piped())
        .stdout(::std::process::Stdio::piped())
        .stderr(::std::process::Stdio::piped());
    ;

    info!("service {} started", &service.command);

    let mut spawn = cmd.spawn().context(format!("could not spawn child {}", &service.command))?;

    let stderr = spawn.stderr.take().context("take stderr")?;
    let jh_stderr = read_stream_to_end("stderr", stderr);

    let stdout = spawn.stdout.take().context("take stdout")?;
    let jh_stdout = read_stream_to_end("stdout", stdout);

    let res: Result<(Result<(), anyhow::Error>, Result<(), anyhow::Error>), JoinError> = try_join!(jh_stderr, jh_stdout);
    debug!("service {} stdout and stderr closed", &service.command);

    let exitcode = spawn.wait().await?;
    info!("service {} process finished with exitcode {:?}", &service.command, &exitcode.code());

    Ok(())
}

fn read_stream_to_end<T>(stream_name: &'static str, stream: T) -> JoinHandle<Result<(), Error>>
    where
        T: AsyncReadExt,
        T: Unpin,
        T: Send,
        T: 'static
{
    ::tokio::spawn(async move {
        let mut reader = BufReader::new(stream);

        let mut buf = Vec::with_capacity(4096);
        loop {
            let size = reader.read_buf(&mut buf).await?;

            if size == 0 {
                debug!("finished reading {}", stream_name);
                break;
            }

            ::std::io::stdout().write_all(&buf[0..size])?;
        }

        Ok(())
    })
}