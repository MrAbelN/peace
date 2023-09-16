use std::{fmt::Debug, marker::PhantomData};

use peace_cmd::{
    ctx::CmdCtx,
    scopes::{SingleProfileSingleFlow, SingleProfileSingleFlowView},
};
use peace_cmd_rt::{CmdBlockWrapper, CmdExecution};
use peace_resources::{
    paths::{FlowDir, StatesCurrentFile},
    resources::ts::SetUp,
    states::{States, StatesCleaned, StatesCleanedDry, StatesCurrent, StatesPrevious},
    Resources,
};
use peace_rt_model::{
    outcomes::CmdOutcome, output::OutputWrite, params::ParamsKeys, Error, ItemGraph, Storage,
};

use crate::{
    cmd_blocks::{
        apply_exec_cmd_block::StatesTsApplyExt, ApplyExecCmdBlock, ApplyStateSyncCheckCmdBlock,
        StatesCleanInsertionCmdBlock, StatesCurrentReadCmdBlock, StatesDiscoverCmdBlock,
    },
    cmds::ApplyStoredStateSync,
};

#[derive(Debug)]
pub struct CleanCmd<E, O, PKeys>(PhantomData<(E, O, PKeys)>);

impl<E, O, PKeys> CleanCmd<E, O, PKeys>
where
    E: std::error::Error + From<Error> + Send + Sync + Unpin + 'static,
    PKeys: ParamsKeys + 'static,
    O: OutputWrite<E>,
{
    /// Conditionally runs [`Item::apply_exec_dry`] for each [`Item`].
    ///
    /// In practice this runs [`Item::apply_check`], and only runs
    /// [`apply_exec_dry`] if execution is required.
    ///
    /// # Design
    ///
    /// The grouping of item functions run for a `Clean` execution to work
    /// is as follows:
    ///
    /// 1. Run [`StatesDiscoverCmd::current`] for all `Item`s in the
    ///   *forward* direction.
    ///
    ///     This populates `resources` with `Current<IS::State>`, needed for
    ///     `Item::try_state_current` during `ItemRt::clean_prepare`.
    ///
    /// 2. In the *reverse* direction, for each `Item` run
    ///    `ItemRt::clean_prepare`, which runs:
    ///
    ///     1. `Item::try_state_current`, which resolves parameters from the
    ///        *current* state.
    ///     2. `Item::state_goal`
    ///     3. `Item::apply_check`
    ///
    /// 3. For `Item`s that return `ApplyCheck::ExecRequired`, run
    ///    `Item::apply_exec_dry`.
    ///
    /// [`apply_exec_dry`]: peace_cfg::Item::apply_exec_dry
    /// [`Item::apply_check`]: peace_cfg::Item::apply_check
    /// [`Item::apply_exec_dry`]: peace_cfg::ItemRt::apply_exec_dry
    /// [`Item`]: peace_cfg::Item
    pub async fn exec_dry(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'_, E, O, PKeys, SetUp>>,
    ) -> Result<CmdOutcome<StatesCleanedDry, E>, E> {
        Self::exec_dry_with(cmd_ctx, ApplyStoredStateSync::Both).await
    }

    /// Conditionally runs [`Item::apply_exec_dry`] for each [`Item`].
    ///
    /// See [`Self::exec_dry`] for full documentation.
    ///
    /// This function exists so that this command can be executed as sub
    /// functionality of another command.
    pub async fn exec_dry_with(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'_, E, O, PKeys, SetUp>>,
        apply_stored_state_sync: ApplyStoredStateSync,
    ) -> Result<CmdOutcome<StatesCleanedDry, E>, E> {
        Self::exec_internal(cmd_ctx, apply_stored_state_sync).await
    }

    /// Conditionally runs [`Item::apply_exec`] for each [`Item`].
    ///
    /// In practice this runs [`Item::apply_check`], and only runs
    /// [`apply_exec`] if execution is required.
    ///
    /// # Design
    ///
    /// The grouping of item functions run for a `Clean` execution to work
    /// is as follows:
    ///
    /// 1. Run [`StatesDiscoverCmd::current`] for all `Item`s in the
    ///   *forward* direction.
    ///
    ///     This populates `resources` with `Current<IS::State>`, needed for
    ///     `Item::try_state_current` during `ItemRt::clean_prepare`.
    ///
    /// 2. In the *reverse* direction, for each `Item` run
    ///    `ItemRt::clean_prepare`, which runs:
    ///
    ///     1. `Item::try_state_current`, which resolves parameters from the
    ///        *current* state.
    ///     2. `Item::state_goal`
    ///     3. `Item::apply_check`
    ///
    /// 3. For `Item`s that return `ApplyCheck::ExecRequired`, run
    ///    `Item::apply_exec`.
    ///
    /// [`apply_exec`]: peace_cfg::Item::apply_exec
    /// [`Item::apply_check`]: peace_cfg::Item::apply_check
    /// [`Item::apply_exec`]: peace_cfg::ItemRt::apply_exec
    /// [`Item`]: peace_cfg::Item
    pub async fn exec(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'_, E, O, PKeys, SetUp>>,
    ) -> Result<CmdOutcome<StatesCleaned, E>, E> {
        Self::exec_with(cmd_ctx, ApplyStoredStateSync::Both).await
    }

    /// Conditionally runs [`Item::apply_exec`] for each [`Item`].
    ///
    /// See [`Self::exec`] for full documentation.
    ///
    /// This function exists so that this command can be executed as sub
    /// functionality of another command.
    pub async fn exec_with(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'_, E, O, PKeys, SetUp>>,
        apply_stored_state_sync: ApplyStoredStateSync,
    ) -> Result<CmdOutcome<StatesCleaned, E>, E> {
        let CmdOutcome {
            value: states_cleaned,
            errors,
        } = Self::exec_internal(cmd_ctx, apply_stored_state_sync).await?;

        let SingleProfileSingleFlowView {
            flow, resources, ..
        } = cmd_ctx.view();
        let (item_graph, resources) = (flow.graph(), resources);

        Self::serialize_current(item_graph, resources, &states_cleaned).await?;

        let cmd_outcome = CmdOutcome {
            value: states_cleaned,
            errors,
        };
        Ok(cmd_outcome)
    }

    /// Conditionally runs [`ApplyFns`]`::`[`exec`] for each [`Item`].
    ///
    /// Same as [`Self::exec`], but does not change the type state, and returns
    /// [`StatesCleaned`].
    ///
    /// [`exec`]: peace_cfg::ApplyFns::exec
    /// [`Item`]: peace_cfg::Item
    /// [`ApplyFns`]: peace_cfg::Item::ApplyFns
    async fn exec_internal<StatesTs>(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'_, E, O, PKeys, SetUp>>,
        apply_stored_state_sync: ApplyStoredStateSync,
    ) -> Result<CmdOutcome<States<StatesTs>, E>, E>
    where
        StatesTs: StatesTsApplyExt + Debug + Send + Sync + Unpin + 'static,
    {
        let mut cmd_execution = {
            let mut cmd_execution_builder =
                CmdExecution::<(StatesPrevious, States<StatesTs>), _, _>::builder()
                    .with_cmd_block(CmdBlockWrapper::new(
                        StatesCurrentReadCmdBlock::new(),
                        |states_current_stored| {
                            (
                                StatesPrevious::from(states_current_stored.into_inner()),
                                States::<StatesTs>::new(),
                            )
                        },
                    ))
                    .with_cmd_block(CmdBlockWrapper::new(
                        StatesCleanInsertionCmdBlock::new(),
                        |_| Default::default(),
                    ));

            cmd_execution_builder = match apply_stored_state_sync {
                ApplyStoredStateSync::None => cmd_execution_builder,
                ApplyStoredStateSync::Current => cmd_execution_builder
                    .with_cmd_block(CmdBlockWrapper::new(
                        StatesDiscoverCmdBlock::current(),
                        |states_current_mut| {
                            (
                                StatesPrevious::from(StatesCurrent::from(states_current_mut)),
                                States::<StatesTs>::new(),
                            )
                        },
                    ))
                    .with_cmd_block(CmdBlockWrapper::new(
                        ApplyStateSyncCheckCmdBlock::current(),
                        |_| Default::default(),
                    )),

                // Data modelling doesn't work well here -- for `CleanCmd` we don't check if the
                // `goal` state is in sync before cleaning, as the target state is `state_clean`
                // instead of `state_goal`.
                ApplyStoredStateSync::Goal => cmd_execution_builder.with_cmd_block(
                    CmdBlockWrapper::new(StatesDiscoverCmdBlock::current(), |states_current_mut| {
                        (
                            StatesPrevious::from(StatesCurrent::from(states_current_mut)),
                            States::<StatesTs>::new(),
                        )
                    }),
                ),

                // Similar to the above, we only discover `state_current` even if both are requested
                // to be in sync.
                ApplyStoredStateSync::Both => cmd_execution_builder
                    .with_cmd_block(CmdBlockWrapper::new(
                        StatesDiscoverCmdBlock::current(),
                        |states_current_mut| {
                            (
                                StatesPrevious::from(StatesCurrent::from(states_current_mut)),
                                States::<StatesTs>::new(),
                            )
                        },
                    ))
                    .with_cmd_block(CmdBlockWrapper::new(
                        ApplyStateSyncCheckCmdBlock::current(),
                        |_| Default::default(),
                    )),
            };

            cmd_execution_builder
                .with_cmd_block(CmdBlockWrapper::new(
                    ApplyExecCmdBlock::<E, PKeys, StatesTs>::new(),
                    |(states_previous, states_applied_mut, _states_target_mut)| {
                        (
                            states_previous,
                            States::<StatesTs>::from(states_applied_mut),
                        )
                    },
                ))
                .with_execution_outcome_fetch(|resources| {
                    let states_previous =
                        resources.remove::<StatesPrevious>().unwrap_or_else(|| {
                            let states_previous = tynm::type_name::<StatesPrevious>();
                            panic!("Expected `{states_previous}` to exist in `Resources`");
                        });
                    let states_cleaned =
                        resources.remove::<States<StatesTs>>().unwrap_or_else(|| {
                            let states_cleaned = tynm::type_name::<States<StatesTs>>();
                            panic!("Expected `{states_cleaned}` to exist in `Resources`");
                        });

                    (states_previous, states_cleaned)
                })
                .build()
        };

        let cmd_outcome = cmd_execution.exec(cmd_ctx).await?;

        // TODO: Should we run `StatesCurrentFn` again?
        //
        // i.e. is it part of `ApplyFns::exec`'s contract to return the state.
        //
        // * It may be duplication of code.
        // * `FileDownloadItem` needs to know the ETag from the last request, which:
        //     - in `StatesCurrentFn` comes from `StatesCurrent`
        //     - in `CleanCmd` comes from `Cleaned`
        // * `ShCmdItem` doesn't return the state in the apply script, so in the item we
        //   run the state current script after the apply exec script.

        let cmd_outcome = cmd_outcome.map(|(states_previous, states_cleaned)| {
            cmd_ctx
                .view()
                .resources
                .insert::<StatesPrevious>(states_previous);

            states_cleaned
        });

        Ok(cmd_outcome)
    }

    // TODO: This duplicates a bit of code with `StatesDiscoverCmd`,
    async fn serialize_current(
        item_graph: &ItemGraph<E>,
        resources: &Resources<SetUp>,
        states_cleaned: &StatesCleaned,
    ) -> Result<(), E> {
        use peace_rt_model::StatesSerializer;

        let flow_dir = resources.borrow::<FlowDir>();
        let storage = resources.borrow::<Storage>();
        let states_current_file = StatesCurrentFile::from(&*flow_dir);

        StatesSerializer::serialize(&storage, item_graph, states_cleaned, &states_current_file)
            .await?;

        drop(flow_dir);
        drop(storage);

        Ok(())
    }
}

impl<E, O, PKeys> Default for CleanCmd<E, O, PKeys> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
