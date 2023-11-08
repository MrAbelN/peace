use std::{fmt::Debug, marker::PhantomData};

use async_trait::async_trait;
use peace_cmd::scopes::SingleProfileSingleFlowView;
use peace_cmd_model::{fn_graph::StreamOutcomeState, CmdBlockDesc, CmdBlockOutcome};
use peace_resources::{resources::ts::SetUp, Resource};
use peace_rt_model::{outcomes::CmdOutcome, params::ParamsKeys};
use tynm::TypeParamsFmtOpts;

use crate::{CmdBlock, CmdBlockError, CmdBlockRt};

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        use peace_cfg::progress::ProgressUpdateAndId;
        use tokio::sync::mpsc::Sender;
    }
}

/// Wraps a [`CmdBlock`] and holds a partial execution handler.
///
/// The following are the technical reasons for this type's existence:
///
/// * Being in the `peace_cmd` crate, the type erased [`CmdBlockRt`] trait can
///   be implemented on this type within this crate.
/// * The partial execution handler specifies how a command execution should
///   finish, if execution is interrupted or there is an error with one item
///   within the flow.
///
/// [`CmdBlockRt`]: crate::CmdBlockRt
#[derive(Debug)]
pub struct CmdBlockWrapper<CB, E, PKeys, ExecutionOutcome, BlockOutcome, InputT> {
    /// Underlying `CmdBlock` implementation.
    ///
    /// The trait constraints are applied on impl blocks.
    cmd_block: CB,
    /// Function to run if interruption or an item failure happens while
    /// executing this `CmdBlock`.
    fn_partial_exec_handler: fn(BlockOutcome) -> ExecutionOutcome,
    /// Marker.
    marker: PhantomData<(E, PKeys, BlockOutcome, InputT)>,
}

impl<CB, E, PKeys, ExecutionOutcome, BlockOutcome, InputT>
    CmdBlockWrapper<CB, E, PKeys, ExecutionOutcome, BlockOutcome, InputT>
where
    CB: CmdBlock<Error = E, PKeys = PKeys, InputT = InputT>,
{
    /// Returns a new `CmdBlockWrapper`.
    ///
    /// # Parameters
    ///
    /// * `cmd_block`: The `CmdBlock` implementation.
    /// * `fn_partial_exec_handler`: How the `CmdExecution` should end, if
    ///   execution ends with this `CmdBlock`.
    ///
    ///     This could be due to interruption, or a `CmdOutcome` with an item
    ///     failure.
    pub fn new(
        cmd_block: CB,
        fn_partial_exec_handler: fn(BlockOutcome) -> ExecutionOutcome,
    ) -> Self {
        Self {
            cmd_block,
            fn_partial_exec_handler,
            marker: PhantomData,
        }
    }
}

#[async_trait(?Send)]
impl<CB, E, PKeys, ExecutionOutcome, BlockOutcome, InputT> CmdBlockRt
    for CmdBlockWrapper<CB, E, PKeys, ExecutionOutcome, BlockOutcome, InputT>
where
    CB: CmdBlock<Error = E, PKeys = PKeys, Outcome = BlockOutcome, InputT = InputT> + Unpin,
    E: Debug + std::error::Error + From<peace_rt_model::Error> + Send + Unpin + 'static,
    PKeys: ParamsKeys + 'static,
    ExecutionOutcome: Debug + Unpin + Send + Sync + 'static,
    BlockOutcome: Debug + Unpin + Send + Sync + 'static,
    InputT: Debug + Resource + Unpin + 'static,
{
    type Error = E;
    type ExecutionOutcome = ExecutionOutcome;
    type PKeys = PKeys;

    async fn exec(
        &self,
        cmd_view: &mut SingleProfileSingleFlowView<'_, Self::Error, Self::PKeys, SetUp>,
        #[cfg(feature = "output_progress")] progress_tx: Sender<ProgressUpdateAndId>,
    ) -> Result<(), CmdBlockError<ExecutionOutcome, Self::Error>> {
        let cmd_block = &self.cmd_block;
        let input = cmd_block.input_fetch(cmd_view.resources)?;

        let cmd_block_outcome = cmd_block
            .exec(
                input,
                cmd_view,
                #[cfg(feature = "output_progress")]
                &progress_tx,
            )
            .await
            .map_err(CmdBlockError::Exec)?;

        // `progress_tx` is dropped here, so `progress_rx` will safely end.
        #[cfg(feature = "output_progress")]
        drop(progress_tx);

        match cmd_block_outcome {
            CmdBlockOutcome::Single(block_outcome) => {
                cmd_block.outcome_insert(cmd_view.resources, block_outcome);
                Ok(())
            }
            CmdBlockOutcome::ItemWise {
                stream_outcome,
                errors,
            } => {
                if errors.is_empty() {
                    match stream_outcome.state {
                        StreamOutcomeState::NotStarted => {
                            let cmd_block_name = tynm::type_name::<CB>();

                            unreachable!(
                                "`{cmd_block_name}` returned `StreamOutcomeState::NotStarted`.\n\
                                This should be impossible as `FnGraph` stream functions always poll their underlying stream.\n\
                                \n\
                                This is a bug, please report it at <https://github.com/azriel91/peace>."
                            );
                        }
                        StreamOutcomeState::Interrupted => {
                            let stream_outcome = stream_outcome.map(self.fn_partial_exec_handler);
                            let cmd_outcome = CmdOutcome::BlockInterrupted { stream_outcome };

                            Err(CmdBlockError::Interrupt(cmd_outcome))
                        }
                        StreamOutcomeState::Finished => {
                            let block_outcome = stream_outcome.value;
                            cmd_block.outcome_insert(cmd_view.resources, block_outcome);

                            Ok(())
                        }
                    }
                } else {
                    // If possible, `CmdBlock` outcomes with item errors need to be mapped to
                    // the `CmdExecution` outcome type, so we still return the item errors.
                    //
                    // e.g. `StatesCurrentMut` should be mapped into `StatesEnsured` when some
                    // items fail to be ensured.
                    //
                    // Note, when discovering current and goal states for diffing, and an item
                    // error occurs, mapping the partially accumulated `(StatesCurrentMut,
                    // StatesGoalMut)` into `StateDiffs` may or may not be semantically
                    // meaningful.

                    let stream_outcome = stream_outcome.map(self.fn_partial_exec_handler);
                    let cmd_outcome = CmdOutcome::ItemError {
                        stream_outcome,
                        errors,
                    };
                    Err(CmdBlockError::ItemError(cmd_outcome))
                }
            }
        }
    }

    fn cmd_block_desc(&self) -> CmdBlockDesc {
        let cmd_block_name = tynm::type_name_opts::<CB>(TypeParamsFmtOpts::Std);
        let cmd_block_input_names = self.cmd_block.input_type_names();
        let cmd_block_outcome_names = self.cmd_block.outcome_type_names();

        CmdBlockDesc::new(
            cmd_block_name,
            cmd_block_input_names,
            cmd_block_outcome_names,
        )
    }
}
