use std::{fmt::Debug, marker::PhantomData};

use peace_cfg::{ApplyCheck, FnCtx, ItemId};
use peace_cmd::scopes::SingleProfileSingleFlowView;
use peace_cmd_rt::{
    async_trait,
    fn_graph::{StreamOpts, StreamOutcome},
    CmdBlock,
};
use peace_params::ParamsSpecs;
use peace_resources::{
    internal::StatesMut,
    resources::ts::SetUp,
    states::{
        ts::{Clean, Cleaned, CleanedDry, Ensured, EnsuredDry, Goal},
        States, StatesCurrent, StatesPrevious,
    },
    ResourceFetchError, Resources,
};
use peace_rt_model::{
    outcomes::{CmdOutcome, ItemApplyBoxed, ItemApplyPartialBoxed},
    params::ParamsKeys,
    Error, ItemBoxed, ItemRt,
};

use tokio::sync::mpsc::UnboundedSender;

use crate::BUFFERED_FUTURES_MAX;

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        use peace_cfg::{
            progress::{
                ProgressComplete,
                ProgressMsgUpdate,
                ProgressUpdate,
                ProgressUpdateAndId,
                ProgressSender,
            },
        };
        use tokio::sync::mpsc::Sender;
    }
}

/// Stops a `CmdExecution` if stored states and discovered states are not in
/// sync.
pub struct ApplyExecCmdBlock<E, PKeys, StatesTs>(PhantomData<(E, PKeys, StatesTs)>);

impl<E, PKeys, StatesTs> Debug for ApplyExecCmdBlock<E, PKeys, StatesTs> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ApplyExecCmdBlock").field(&self.0).finish()
    }
}

impl<E, PKeys, StatesTs> ApplyExecCmdBlock<E, PKeys, StatesTs> {
    /// Returns an `ApplyExecCmdBlock`.
    ///
    /// This is a generic constructor where `StatesTs` determines whether the
    /// goal state or clean state is the target state.
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<E, PKeys, StatesTs> Default for ApplyExecCmdBlock<E, PKeys, StatesTs> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<E, PKeys> ApplyExecCmdBlock<E, PKeys, Ensured>
where
    E: std::error::Error + From<Error> + Send + 'static,
    PKeys: ParamsKeys + 'static,
{
    /// Returns an `ApplyExecCmdBlock` with the goal state as the target state.
    pub fn ensure() -> Self {
        Self(PhantomData)
    }
}

impl<E, PKeys> ApplyExecCmdBlock<E, PKeys, EnsuredDry>
where
    E: std::error::Error + From<Error> + Send + 'static,
    PKeys: ParamsKeys + 'static,
{
    /// Returns an `ApplyExecCmdBlock` with the goal state as the target state.
    pub fn ensure_dry() -> Self {
        Self(PhantomData)
    }
}

impl<E, PKeys> ApplyExecCmdBlock<E, PKeys, Cleaned>
where
    E: std::error::Error + From<Error> + Send + 'static,
    PKeys: ParamsKeys + 'static,
{
    /// Returns an `ApplyExecCmdBlock` with the clean state as the target state.
    pub fn clean() -> Self {
        Self(PhantomData)
    }
}

impl<E, PKeys> ApplyExecCmdBlock<E, PKeys, CleanedDry>
where
    E: std::error::Error + From<Error> + Send + 'static,
    PKeys: ParamsKeys + 'static,
{
    /// Returns an `ApplyExecCmdBlock` with the clean state as the target state.
    pub fn clean_dry() -> Self {
        Self(PhantomData)
    }
}

impl<E, PKeys, StatesTs> ApplyExecCmdBlock<E, PKeys, StatesTs>
where
    PKeys: ParamsKeys + 'static,
    StatesTs: StatesTsApplyExt + Debug + Send,
    E: std::error::Error + 'static,
{
    ///
    /// # Implementation Note
    ///
    /// Tried passing through the function to execute instead of a `dry_run`
    /// parameter, but couldn't convince the compiler that the lifetimes match
    /// up:
    ///
    /// ```rust,ignore
    /// async fn item_apply_exec<F, Fut>(
    ///     resources: &Resources<SetUp>,
    ///     outcomes_tx: &UnboundedSender<ItemApplyOutcome<E>>,
    ///     item: FnRef<'_, ItemBoxed<E>>,
    ///     f: F,
    /// ) -> bool
    /// where
    ///     F: (Fn(&dyn ItemRt<E>, fn_ctx: OpCtx<'_>, &Resources<SetUp>, &mut ItemApplyBoxed) -> Fut) + Copy,
    ///     Fut: Future<Output = Result<(), E>>,
    /// ```
    async fn item_apply_exec(
        item_apply_exec_ctx: ItemApplyExecCtx<'_, E>,
        item: &ItemBoxed<E>,
    ) -> Result<(), ()> {
        let ItemApplyExecCtx {
            params_specs,
            resources,
            apply_for_internal,
            #[cfg(feature = "output_progress")]
            progress_tx,
            outcomes_tx,
        } = item_apply_exec_ctx;

        let apply_fn = if StatesTs::dry_run() {
            ItemRt::apply_exec_dry
        } else {
            ItemRt::apply_exec
        };

        let item_id = item.id();
        let fn_ctx = FnCtx::new(
            item_id,
            #[cfg(feature = "output_progress")]
            ProgressSender::new(item_id, progress_tx),
        );
        let item_apply = match apply_for_internal {
            ApplyForInternal::Ensure => {
                ItemRt::ensure_prepare(&**item, params_specs, resources, fn_ctx).await
            }
            ApplyForInternal::Clean { states_current } => {
                ItemRt::clean_prepare(&**item, states_current, params_specs, resources).await
            }
        };

        match item_apply {
            Ok(mut item_apply) => {
                match item_apply.apply_check() {
                    #[cfg(not(feature = "output_progress"))]
                    ApplyCheck::ExecRequired => {}
                    #[cfg(feature = "output_progress")]
                    ApplyCheck::ExecRequired { progress_limit } => {
                        // Update `OutputWrite`s with progress limit.
                        let _progress_send_unused = progress_tx.try_send(ProgressUpdateAndId {
                            item_id: item_id.clone(),
                            progress_update: ProgressUpdate::Limit(progress_limit),
                            msg_update: ProgressMsgUpdate::Set(String::from("in progress")),
                        });
                    }
                    ApplyCheck::ExecNotRequired => {
                        #[cfg(feature = "output_progress")]
                        let _progress_send_unused = progress_tx.try_send(ProgressUpdateAndId {
                            item_id: item_id.clone(),
                            progress_update: ProgressUpdate::Complete(ProgressComplete::Success),
                            msg_update: ProgressMsgUpdate::Set(String::from("nothing to do!")),
                        });

                        // TODO: write test for this case
                        // In case of an interrupt or power failure, we may not have written states
                        // to disk.
                        outcomes_tx
                            .send(ItemApplyOutcome::Success {
                                item_id: item.id().clone(),
                                item_apply,
                            })
                            .expect("unreachable: `outcomes_rx` is in a sibling task.");

                        // short-circuit
                        return Ok(());
                    }
                }
                match apply_fn(&**item, params_specs, resources, fn_ctx, &mut item_apply).await {
                    Ok(()) => {
                        // apply succeeded

                        #[cfg(feature = "output_progress")]
                        let _progress_send_unused = progress_tx.try_send(ProgressUpdateAndId {
                            item_id: item_id.clone(),
                            progress_update: ProgressUpdate::Complete(ProgressComplete::Success),
                            msg_update: ProgressMsgUpdate::Set(String::from("done!")),
                        });

                        outcomes_tx
                            .send(ItemApplyOutcome::Success {
                                item_id: item.id().clone(),
                                item_apply,
                            })
                            .expect("unreachable: `outcomes_rx` is in a sibling task.");

                        Ok(())
                    }
                    Err(error) => {
                        // apply failed

                        #[cfg(feature = "output_progress")]
                        let _progress_send_unused = progress_tx.try_send(ProgressUpdateAndId {
                            item_id: item_id.clone(),
                            progress_update: ProgressUpdate::Complete(ProgressComplete::Fail),
                            msg_update: ProgressMsgUpdate::Set(
                                error
                                    .source()
                                    .map(|source| format!("{source}"))
                                    .unwrap_or_else(|| format!("{error}")),
                            ),
                        });

                        outcomes_tx
                            .send(ItemApplyOutcome::Fail {
                                item_id: item.id().clone(),
                                item_apply,
                                error,
                            })
                            .expect("unreachable: `outcomes_rx` is in a sibling task.");

                        // we should stop processing.
                        Err(())
                    }
                }
            }
            Err((error, item_apply_partial)) => {
                #[cfg(feature = "output_progress")]
                let _progress_send_unused = progress_tx.try_send(ProgressUpdateAndId {
                    item_id: item.id().clone(),
                    progress_update: ProgressUpdate::Complete(ProgressComplete::Fail),
                    msg_update: ProgressMsgUpdate::Set(
                        error
                            .source()
                            .map(|source| format!("{source}"))
                            .unwrap_or_else(|| format!("{error}")),
                    ),
                });

                outcomes_tx
                    .send(ItemApplyOutcome::PrepareFail {
                        item_id: item.id().clone(),
                        item_apply_partial,
                        error,
                    })
                    .expect("unreachable: `outcomes_rx` is in a sibling task.");

                Err(())
            }
        }
    }
}

#[async_trait(?Send)]
impl<E, PKeys, StatesTs> CmdBlock for ApplyExecCmdBlock<E, PKeys, StatesTs>
where
    E: std::error::Error + From<Error> + Send + 'static,
    PKeys: ParamsKeys + 'static,
    StatesTs: StatesTsApplyExt + Debug + Send + Sync + 'static,
{
    type Error = E;
    type InputT = (StatesCurrent, States<StatesTs::TsTarget>);
    type Outcome = (StatesPrevious, States<StatesTs>, States<StatesTs::TsTarget>);
    type OutcomeAcc = (
        StatesPrevious,
        StatesMut<StatesTs>,
        StatesMut<StatesTs::TsTarget>,
    );
    type OutcomePartial = ItemApplyOutcome<E>;
    type PKeys = PKeys;

    fn input_fetch(
        &self,
        resources: &mut Resources<SetUp>,
    ) -> Result<Self::InputT, ResourceFetchError> {
        let states_current = resources.try_remove::<StatesCurrent>()?;

        let states_target = resources.try_remove::<States<StatesTs::TsTarget>>()?;

        Ok((states_current, states_target))
    }

    fn input_type_names(&self) -> Vec<String> {
        vec![
            tynm::type_name::<StatesCurrent>(),
            tynm::type_name::<States<StatesTs::TsTarget>>(),
        ]
    }

    fn outcome_acc_init(&self, input: &Self::InputT) -> Self::OutcomeAcc {
        let (states_current, states_target) = input;
        (
            StatesPrevious::from(states_current.clone()),
            // `Ensured`, `EnsuredDry`, `Cleaned`, `CleanedDry` states start as the current state,
            // and are altered.
            StatesMut::<StatesTs>::from(states_current.clone().into_inner()),
            StatesMut::<StatesTs::TsTarget>::from(states_target.clone().into_inner()),
        )
    }

    fn outcome_from_acc(&self, outcome_acc: Self::OutcomeAcc) -> Self::Outcome {
        let (states_previous, states_applied_mut, states_target_mut) = outcome_acc;

        (
            states_previous,
            States::<StatesTs>::from(states_applied_mut),
            States::<StatesTs::TsTarget>::from(states_target_mut),
        )
    }

    fn outcome_insert(&self, resources: &mut Resources<SetUp>, outcome: Self::Outcome) {
        let (states_previous, states_applied, states_target) = outcome;
        resources.insert(states_previous);
        resources.insert(states_applied);
        resources.insert(states_target);
    }

    fn outcome_type_names(&self) -> Vec<String> {
        vec![
            tynm::type_name::<StatesPrevious>(),
            tynm::type_name::<States<StatesTs>>(),
            tynm::type_name::<States<StatesTs::TsTarget>>(),
        ]
    }

    async fn exec(
        &self,
        input: Self::InputT,
        cmd_view: &mut SingleProfileSingleFlowView<'_, Self::Error, Self::PKeys, SetUp>,
        outcomes_tx: &UnboundedSender<Self::OutcomePartial>,
        #[cfg(feature = "output_progress")] progress_tx: &Sender<ProgressUpdateAndId>,
    ) -> Option<StreamOutcome<()>> {
        let (states_current, _states_target) = input;

        let SingleProfileSingleFlowView {
            interruptibility,
            flow,
            params_specs,
            resources,
            ..
        } = cmd_view;

        let item_graph = flow.graph();
        let resources_ref = &*resources;
        let apply_for = StatesTs::apply_for();
        let apply_for_internal = match apply_for {
            ApplyFor::Ensure => ApplyForInternal::Ensure,
            ApplyFor::Clean => ApplyForInternal::Clean { states_current },
        };

        match apply_for {
            ApplyFor::Ensure => {
                let (Ok(stream_outcome) | Err((stream_outcome, ()))) = item_graph
                    .try_for_each_concurrent_with(
                        BUFFERED_FUTURES_MAX,
                        StreamOpts::new().interruptibility(interruptibility.reborrow()),
                        |item| {
                            let item_apply_exec_ctx = ItemApplyExecCtx {
                                params_specs,
                                resources: resources_ref,
                                apply_for_internal: &apply_for_internal,
                                #[cfg(feature = "output_progress")]
                                progress_tx,
                                outcomes_tx,
                            };
                            Self::item_apply_exec(item_apply_exec_ctx, item)
                        },
                    )
                    .await
                    .map_err(|(outcome, _vec_units): (StreamOutcome<()>, Vec<()>)| (outcome, ()));

                Some(stream_outcome)
            }
            ApplyFor::Clean => {
                let (Ok(stream_outcome) | Err((stream_outcome, ()))) = item_graph
                    .try_for_each_concurrent_with(
                        BUFFERED_FUTURES_MAX,
                        StreamOpts::new()
                            .rev()
                            .interruptibility(interruptibility.reborrow()),
                        |item| {
                            let item_apply_exec_ctx = ItemApplyExecCtx {
                                params_specs,
                                resources: resources_ref,
                                apply_for_internal: &apply_for_internal,
                                #[cfg(feature = "output_progress")]
                                progress_tx,
                                outcomes_tx,
                            };
                            Self::item_apply_exec(item_apply_exec_ctx, item)
                        },
                    )
                    .await
                    .map_err(|(outcome, _vec_units): (StreamOutcome<()>, Vec<()>)| (outcome, ()));

                Some(stream_outcome)
            }
        }
    }

    fn outcome_collate(
        &self,
        block_outcome: &mut CmdOutcome<Self::OutcomeAcc, Self::Error>,
        outcome_partial: Self::OutcomePartial,
    ) -> Result<(), Self::Error> {
        let CmdOutcome {
            value: (_states_previous, states_applied_mut, states_target_mut),
            errors,
        } = block_outcome;

        let apply_for = StatesTs::apply_for();

        match outcome_partial {
            ItemApplyOutcome::PrepareFail {
                item_id,
                item_apply_partial,
                error,
            } => {
                errors.insert(item_id.clone(), error);

                // Save `state_target` (which is `state_goal`) if we are not cleaning
                // up.
                match apply_for {
                    ApplyFor::Ensure => {
                        if let Some(state_target) = item_apply_partial.state_target() {
                            states_target_mut.insert_raw(item_id, state_target);
                        }
                    }
                    ApplyFor::Clean => {}
                }
            }
            ItemApplyOutcome::Success {
                item_id,
                item_apply,
            } => {
                if let Some(state_applied) = item_apply.state_applied() {
                    states_applied_mut.insert_raw(item_id.clone(), state_applied);
                } else {
                    // Item was already in the goal state.
                    // No change to current state.
                }

                // Save `state_target` (which is state_target) if we are not cleaning
                // up.
                match apply_for {
                    ApplyFor::Ensure => {
                        let state_target = item_apply.state_target();
                        states_target_mut.insert_raw(item_id, state_target);
                    }
                    ApplyFor::Clean => {}
                }
            }
            ItemApplyOutcome::Fail {
                item_id,
                item_apply,
                error,
            } => {
                errors.insert(item_id.clone(), error);
                if let Some(state_applied) = item_apply.state_applied() {
                    states_applied_mut.insert_raw(item_id.clone(), state_applied);
                }

                // Save `state_target` (which is state_target) if we are not cleaning
                // up.
                match apply_for {
                    ApplyFor::Ensure => {
                        let state_target = item_apply.state_target();
                        states_target_mut.insert_raw(item_id, state_target);
                    }
                    ApplyFor::Clean => {}
                }
            }
        }

        Ok(())
    }
}

/// Whether the `ApplyCmd` is for `Ensure` or `Clean`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyFor {
    /// The apply target state is `state_goal`.
    Ensure,
    /// The apply target state is `state_clean`.
    Clean,
}

/// Whether the `ApplyCmd` is for `Ensure` or `Clean`.
#[derive(Debug)]
enum ApplyForInternal {
    Ensure,
    Clean { states_current: StatesCurrent },
}

struct ItemApplyExecCtx<'f, E> {
    /// Map of item ID to its params' specs.
    params_specs: &'f ParamsSpecs,
    /// Map of all types at runtime.
    resources: &'f Resources<SetUp>,
    /// Whether the `ApplyCmd` is for `Ensure` or `Clean`.
    apply_for_internal: &'f ApplyForInternal,
    /// Channel sender for `CmdBlock` item outcomes.
    #[cfg(feature = "output_progress")]
    progress_tx: &'f Sender<ProgressUpdateAndId>,
    outcomes_tx: &'f UnboundedSender<ItemApplyOutcome<E>>,
}

#[derive(Debug)]
pub enum ItemApplyOutcome<E> {
    /// Error occurred when discovering current state, goal states, state
    /// diff, or `ApplyCheck`.
    PrepareFail {
        item_id: ItemId,
        item_apply_partial: ItemApplyPartialBoxed,
        error: E,
    },
    /// Ensure execution succeeded.
    Success {
        item_id: ItemId,
        item_apply: ItemApplyBoxed,
    },
    /// Ensure execution failed.
    Fail {
        item_id: ItemId,
        item_apply: ItemApplyBoxed,
        error: E,
    },
}

/// Infers the target state, ensure or clean, and dry run, from a `StateTs`.
pub trait StatesTsApplyExt {
    type TsTarget: Debug + Send + Sync + Unpin + 'static;

    /// Returns the `ApplyFor` this `StatesTs` is meant for.
    fn apply_for() -> ApplyFor;
    /// Returns whether this `StatesTs` is for a dry run.
    fn dry_run() -> bool;
}

impl StatesTsApplyExt for Ensured {
    type TsTarget = Goal;

    fn apply_for() -> ApplyFor {
        ApplyFor::Ensure
    }

    fn dry_run() -> bool {
        false
    }
}

impl StatesTsApplyExt for EnsuredDry {
    type TsTarget = Goal;

    fn apply_for() -> ApplyFor {
        ApplyFor::Ensure
    }

    fn dry_run() -> bool {
        true
    }
}

impl StatesTsApplyExt for Cleaned {
    type TsTarget = Clean;

    fn apply_for() -> ApplyFor {
        ApplyFor::Clean
    }

    fn dry_run() -> bool {
        false
    }
}

impl StatesTsApplyExt for CleanedDry {
    type TsTarget = Clean;

    fn apply_for() -> ApplyFor {
        ApplyFor::Clean
    }

    fn dry_run() -> bool {
        true
    }
}
