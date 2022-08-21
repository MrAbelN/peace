use clap::Parser;
use peace::resources::Resources;
use tokio::io;

pub use download::{
    desired, diff, ensure, ensure_dry, setup_graph, status, DownloadArgs, DownloadCleanOpSpec,
    DownloadCommand, DownloadEnsureOpSpec, DownloadError, DownloadItemSpec, DownloadParams,
    DownloadStateCurrentFnSpec, DownloadStateDesiredFnSpec, DownloadStateDiffFnSpec, FileState,
    FileStateDiff,
};

pub fn main() -> Result<(), DownloadError> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .thread_name("main")
        .thread_stack_size(3 * 1024 * 1024)
        .enable_io()
        .enable_time()
        .build()
        .map_err(DownloadError::TokioRuntimeInit)?;

    let DownloadArgs { command } = DownloadArgs::parse();
    runtime.block_on(async {
        match command {
            DownloadCommand::Status { url, dest } => {
                let graph = setup_graph(url, dest).await?;
                let resources = graph.setup(Resources::new()).await?;
                status(io::stdout(), &graph, resources).await?;
            }
            DownloadCommand::Desired { url, dest } => {
                let graph = setup_graph(url, dest).await?;
                let resources = graph.setup(Resources::new()).await?;
                desired(io::stdout(), &graph, resources).await?;
            }
            DownloadCommand::Diff { url, dest } => {
                let graph = setup_graph(url, dest).await?;
                let resources = graph.setup(Resources::new()).await?;
                diff(io::stdout(), &graph, resources).await?;
            }
            DownloadCommand::EnsureDry { url, dest } => {
                let graph = setup_graph(url, dest).await?;
                let resources = graph.setup(Resources::new()).await?;
                ensure_dry(io::stdout(), &graph, resources).await?;
            }
            DownloadCommand::Ensure { url, dest } => {
                let graph = setup_graph(url, dest).await?;
                let resources = graph.setup(Resources::new()).await?;
                ensure(io::stdout(), &graph, resources).await?;
            }
        }

        Ok::<_, DownloadError>(())
    })
}