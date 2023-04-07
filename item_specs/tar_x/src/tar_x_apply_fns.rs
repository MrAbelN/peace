use std::marker::PhantomData;

#[cfg(feature = "output_progress")]
use peace::cfg::progress::ProgressLimit;
use peace::cfg::{OpCheckStatus, OpCtx};

use crate::{FileMetadatas, TarXData, TarXError, TarXStateDiff};

/// ApplyFns for the tar to extract.
#[derive(Debug)]
pub struct TarXApplyFns<Id>(PhantomData<Id>);

impl<Id> TarXApplyFns<Id>
where
    Id: Send + Sync + 'static,
{
    // Not sure why we can't use this:
    //
    // #[cfg(not(feature = "output_progress"))] _state_desired: &FileMetadatas,
    // #[cfg(feature = "output_progress")] state_desired: &FileMetadatas,
    //
    // There's an error saying lifetime bounds don't match the trait definition.
    //
    // Likely an issue with the codegen in `async-trait`.
    #[allow(unused_variables)]
    pub async fn apply_check(
        _tar_x_data: TarXData<'_, Id>,
        _state_current: &FileMetadatas,
        state_desired: &FileMetadatas,
        diff: &TarXStateDiff,
    ) -> Result<OpCheckStatus, TarXError> {
        let op_check_status = match diff {
            TarXStateDiff::ExtractionInSync => OpCheckStatus::ExecNotRequired,
            TarXStateDiff::ExtractionOutOfSync {
                added: _,
                modified: _,
                removed: _,
            } => {
                #[cfg(not(feature = "output_progress"))]
                {
                    OpCheckStatus::ExecRequired
                }
                #[cfg(feature = "output_progress")]
                {
                    let progress_limit = state_desired
                        .len()
                        .try_into()
                        .map(ProgressLimit::Steps)
                        .unwrap_or(ProgressLimit::Unknown);
                    OpCheckStatus::ExecRequired { progress_limit }
                }
            }
        };

        Ok(op_check_status)
    }

    pub async fn apply_dry(
        _op_ctx: OpCtx<'_>,
        _tar_x_data: TarXData<'_, Id>,
        _state_current: &FileMetadatas,
        state_desired: &FileMetadatas,
        _diff: &TarXStateDiff,
    ) -> Result<FileMetadatas, TarXError> {
        Ok(state_desired.clone())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn apply(
        _op_ctx: OpCtx<'_>,
        tar_x_data: TarXData<'_, Id>,
        _state_current: &FileMetadatas,
        state_desired: &FileMetadatas,
        diff: &TarXStateDiff,
    ) -> Result<FileMetadatas, TarXError> {
        use futures::stream::{StreamExt, TryStreamExt};

        let storage = tar_x_data.storage();
        let params = tar_x_data.tar_x_params();
        let tar_path = params.tar_path();
        let dest = params.dest();

        tokio::fs::create_dir_all(dest)
            .await
            .map_err(|error| TarXError::TarDestDirCreate {
                dest: dest.to_path_buf(),
                error,
            })?;

        // TODO: Optimize by unpacking only the entries that changed.
        // Probably store entries in `IndexMap`s, then look them up to determine if they
        // need to be unpacked.
        //
        // Then we can send proper progress updates via `op_ctx.progress_tx`.
        if tar_path.exists() {
            storage
                .read_with_sync_api(
                    "TarXApplyFns::exec".to_string(),
                    tar_path,
                    |sync_io_bridge| {
                        tar::Archive::new(sync_io_bridge)
                            .unpack(dest)
                            .map_err(|error| TarXError::TarUnpack {
                                tar_path: tar_path.to_path_buf(),
                                dest: dest.to_path_buf(),
                                error,
                            })?;
                        Result::<_, TarXError>::Ok(())
                    },
                )
                .await?;
        }

        if let TarXStateDiff::ExtractionOutOfSync {
            added: _,
            modified: _,
            removed,
        } = diff
        {
            // Remove files that are not in the tar, but are in the destination directory.
            futures::stream::iter(removed.iter())
                .map(|file_metadata| Result::<_, TarXError>::Ok(file_metadata.path()))
                .try_for_each_concurrent(None, |entry_path| async move {
                    tokio::fs::remove_file(&dest.join(entry_path))
                        .await
                        .map_err(|error| TarXError::TarDestFileRemove {
                            dest: dest.to_path_buf(),
                            entry_path: entry_path.to_path_buf(),
                            error,
                        })
                })
                .await?;
        }

        Ok(state_desired.clone())
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn apply(
        _op_ctx: OpCtx<'_>,
        _tar_x_data: TarXData<'_, Id>,
        _state_current: &FileMetadatas,
        _state_desired: &FileMetadatas,
        _diff: &TarXStateDiff,
    ) -> Result<FileMetadatas, TarXError> {
        todo!()
    }
}