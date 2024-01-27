use std::{collections::VecDeque, fmt::Debug};

use peace_cmd::ctx::CmdCtxTypeParamsConstrained;
use peace_resources::{resources::ts::SetUp, Resource, Resources};

use crate::{CmdBlock, CmdBlockRtBox, CmdBlockWrapper, CmdExecution};

/// Collects the [`CmdBlock`]s to run in a `*Cmd` to build a [`CmdExecution`].
///
/// [`CmdBlock`]: crate::CmdBlock
/// [`CmdExecution`]: crate::CmdExecution
#[derive(Debug)]
pub struct CmdExecutionBuilder<'ctx, ExecutionOutcome, CmdCtxTypeParamsT>
where
    ExecutionOutcome: Debug + Send + Sync + 'static,
    CmdCtxTypeParamsT: CmdCtxTypeParamsConstrained,
{
    /// Blocks of commands to run.
    cmd_blocks: VecDeque<CmdBlockRtBox<'ctx, CmdCtxTypeParamsT, ExecutionOutcome>>,
    /// Logic to extract the `ExecutionOutcome` from `Resources`.
    execution_outcome_fetch: fn(&mut Resources<SetUp>) -> Option<ExecutionOutcome>,
    /// Whether or not to render progress.
    ///
    /// This is intended for `*Cmd`s that do not have meaningful progress to
    /// render, such as deserializing a single file on disk, and there is no
    /// benefit to presenting empty progress bars for each item to the user
    ///
    /// Defaults to `true`.
    #[cfg(feature = "output_progress")]
    progress_render_enabled: bool,
}

impl<'ctx, ExecutionOutcome, CmdCtxTypeParamsT>
    CmdExecutionBuilder<'ctx, ExecutionOutcome, CmdCtxTypeParamsT>
where
    ExecutionOutcome: Debug + Send + Sync + 'static,
    CmdCtxTypeParamsT: CmdCtxTypeParamsConstrained + 'ctx,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a `CmdBlock` to this execution.
    pub fn with_cmd_block<CB, BlockOutcomeNext, InputT>(
        self,
        cmd_block: CmdBlockWrapper<
            CB,
            CmdCtxTypeParamsT,
            ExecutionOutcome,
            BlockOutcomeNext,
            InputT,
        >,
    ) -> CmdExecutionBuilder<'ctx, ExecutionOutcome, CmdCtxTypeParamsT>
    where
        CB: CmdBlock<
                CmdCtxTypeParams = CmdCtxTypeParamsT,
                Outcome = BlockOutcomeNext,
                InputT = InputT,
            > + Unpin
            + 'ctx,
        ExecutionOutcome: Debug + Resource + Unpin + 'static,
        BlockOutcomeNext: Debug + Resource + Unpin + 'static,
        InputT: Debug + Resource + Unpin + 'static,
    {
        let CmdExecutionBuilder {
            mut cmd_blocks,
            execution_outcome_fetch,
            #[cfg(feature = "output_progress")]
            progress_render_enabled,
        } = self;

        cmd_blocks.push_back(Box::pin(cmd_block));

        CmdExecutionBuilder {
            cmd_blocks,
            execution_outcome_fetch,
            #[cfg(feature = "output_progress")]
            progress_render_enabled,
        }
    }

    /// Specifies the logic to fetch the `ExecutionOutcome` from `Resources`.
    ///
    /// By default, the `CmdExecution` will run
    /// `resources.remove::<ExecutionOutcome>()`. However, if the
    /// `ExecutionOutcome` is not inserted as a single type, this allows
    /// consumers to specify which types to remove from `resources` and return
    /// as the `ExecutionOutcome`.
    pub fn with_execution_outcome_fetch(
        mut self,
        execution_outcome_fetch: fn(&mut Resources<SetUp>) -> Option<ExecutionOutcome>,
    ) -> Self {
        self.execution_outcome_fetch = execution_outcome_fetch;
        self
    }

    /// Specifies whether or not to render progress.
    ///
    /// This is `true` by default, so usually this would be called with `false`.
    ///
    /// This is intended for `*Cmd`s that do not have meaningful progress to
    /// render, such as deserializing a single file on disk, and there is no
    /// benefit to presenting empty progress bars for each item to the user.
    ///
    /// When this method is called multiple times, the last call wins.
    #[cfg(feature = "output_progress")]
    pub fn with_progress_render_enabled(mut self, progress_render_enabled: bool) -> Self {
        self.progress_render_enabled = progress_render_enabled;
        self
    }

    /// Returns the `CmdExecution` to execute.
    pub fn build(self) -> CmdExecution<'ctx, ExecutionOutcome, CmdCtxTypeParamsT>
    where
        CmdCtxTypeParamsT: CmdCtxTypeParamsConstrained,
    {
        let CmdExecutionBuilder {
            cmd_blocks,
            execution_outcome_fetch,
            #[cfg(feature = "output_progress")]
            progress_render_enabled,
        } = self;

        CmdExecution {
            cmd_blocks,
            execution_outcome_fetch,
            #[cfg(feature = "output_progress")]
            progress_render_enabled,
        }
    }
}

impl<'ctx, ExecutionOutcome, CmdCtxTypeParamsT> Default
    for CmdExecutionBuilder<'ctx, ExecutionOutcome, CmdCtxTypeParamsT>
where
    ExecutionOutcome: Debug + Resource + 'static,
    CmdCtxTypeParamsT: CmdCtxTypeParamsConstrained,
{
    fn default() -> Self {
        Self {
            cmd_blocks: VecDeque::new(),
            execution_outcome_fetch,
            #[cfg(feature = "output_progress")]
            progress_render_enabled: true,
        }
    }
}

fn execution_outcome_fetch<ExecutionOutcome>(
    resources: &mut Resources<SetUp>,
) -> Option<ExecutionOutcome>
where
    ExecutionOutcome: Debug + Send + Sync + 'static,
{
    resources.try_remove::<ExecutionOutcome>().ok()
}
