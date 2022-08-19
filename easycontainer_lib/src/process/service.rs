use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;
use anyhow::{anyhow, Context, Error};
use futures::{join, try_join};
use futures::future::{join_all, try_join_all};
use tokio::io::{AsyncReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::task::{JoinError, JoinHandle};
use tracing::field::debug;
use crate::process::Service;
use tracing::{debug, info};

pub async fn create_services_from_glob(glob_pattern: &str) -> Result<Vec<Service>, ::anyhow::Error> {
    let mut services = vec![];
    for entry in ::glob::glob(glob_pattern).context("Failed to read glob pattern")? {
        let path = &entry?;
        debug!("found service {}", &path.to_string_lossy());
        services.push(create_service_from_file(&path).await.context("create service from file")?);
    }

    debug!("found {} services.", services.len());

    Ok(services)
}

pub async fn run_services_from_glob(glob_pattern: &str) -> Result<(), ::anyhow::Error> {
    let services = create_services_from_glob(glob_pattern).await.context("create_services_from_glob")?;

    let service_runners = {
        let runners: Result<Vec<ServiceRunner>, ::anyhow::Error> = services
            .iter()
            .map(|s| ServiceRunner::new(s.clone()))
            .collect::<_>();

        runners.context("could not create all service runners.")?
    };

    let mut service_handles = service_runners
        .iter()
        .map(|s| s.create_handle())
        .collect::<Vec<_>>();

    let running_services = service_runners
        .into_iter()
        .map(|s| ::tokio::spawn(async move { s.run().await }))
        .collect::<Vec<_>>();


    let mut service_handles_wait_for_all = service_handles.clone();
    let mut wait_for_all = ::tokio::spawn(async move {
        let (result, _, jhs) = ::futures::future::select_all(running_services).await;
        info!("service failed with error {:?}", result);

        for service_handle in &mut service_handles_wait_for_all {
            service_handle.send_kill();
        }

        join_all(jhs).await;

        info!("no service is running anymore.");
    });

    loop {
        ::tokio::select! {
            _ = &mut wait_for_all => {
                info!("all services finished.");
                break;
            },
            _ = ::tokio::signal::ctrl_c() => {
                info!("ctrl_c pressed");

                for service_handle in &mut service_handles {
                    service_handle.send_kill();
                }
            }
        }
    }


    Ok(())
}

pub async fn create_service_from_file(path: &PathBuf) -> Result<Service, ::anyhow::Error> {
    let mut file = tokio::fs::File::open(path)
        .await
        .context(format!("could not open path {}", path.display()))
        ?;

    let mut buf = vec![];
    file.read_to_end(&mut buf).await.context(format!("could not read file {}", path.display())).context("reading file")?;

    let service = toml::from_slice::<Service>(&buf)
        .context(format!("could not parse toml {}", path.display())).context("toml")?;

    Ok(service)
}

struct ServiceRunJh();

#[derive(Clone)]
pub struct ServiceRunHandle {
    service: Service,
    signal_sender: UnboundedSender<()>,
}

impl ServiceRunHandle {
    pub fn send_kill(&mut self) {
        self.signal_sender.send(());
    }
}

pub struct ServiceRunner {
    service: Service,
    jh_stdout: JoinHandle<Result<(), ::anyhow::Error>>,
    jh_stderr: JoinHandle<Result<(), ::anyhow::Error>>,
    spawn: Child,
    signal_receiver: UnboundedReceiver<()>,
    signal_sender: UnboundedSender<()>,
}

impl ServiceRunner {
    pub fn new(service: Service) -> Result<Self, anyhow::Error> {
        let mut cmd = Command::new(service.command.clone());
        let cmd = cmd
            .args(&service.args)
            .stdin(::std::process::Stdio::piped())
            .stdout(::std::process::Stdio::piped())
            .stderr(::std::process::Stdio::piped());
        ;

        let mut spawn = cmd.spawn().context(format!("could not spawn child {}", &service.command))?;

        let stderr = spawn.stderr.take().context("take stderr")?;
        let jh_stderr = read_stream_to_end("stderr", stderr);

        let stdout = spawn.stdout.take().context("take stdout")?;
        let jh_stdout = read_stream_to_end("stdout", stdout);

        let (signal_sender, signal_receiver) = unbounded_channel();

        Ok(
            Self {
                service,
                spawn,
                jh_stderr,
                jh_stdout,
                signal_receiver,
                signal_sender,
            }
        )
    }

    pub fn create_handle(&self) -> ServiceRunHandle {
        ServiceRunHandle {
            signal_sender: self.signal_sender.clone(),
            service: self.service.clone(),
        }
    }

    pub async fn run(mut self) {
        loop {
            let wait = self.spawn.wait();

            ::tokio::select! {
                child = wait => {
                    info!("process finished");
                    break;
                },
                msg = self.signal_receiver.recv() => {
                    info!("start killing service {}", self.service.command);
                    self.spawn.kill().await;
                    info!("</ start killing service {}", self.service.command);
                }
            }
        }

        info!("waiting for pipes.");
        self.jh_stderr.await;
        self.jh_stdout.await;

        info!("service finished.");
    }
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
            let size = reader.read_buf(&mut buf).await.context("readbuf")?;

            if size == 0 {
                debug!("finished reading {}", stream_name);
                break;
            }

            ::std::io::stdout().write_all(&buf[0..size]).context("stdout write")?;
        }

        Ok(())
    })
}