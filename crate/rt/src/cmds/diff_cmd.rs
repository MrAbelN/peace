use std::{fmt::Debug, marker::PhantomData};

use futures::{StreamExt, TryStreamExt};
use peace_cfg::Profile;
use peace_cmd::{
    ctx::CmdCtx,
    scopes::{
        MultiProfileSingleFlow, MultiProfileSingleFlowView, SingleProfileSingleFlow,
        SingleProfileSingleFlowView,
    },
};
use peace_params::ParamsSpecs;
use peace_resources::{
    internal::StateDiffsMut,
    resources::ts::SetUp,
    states::{StateDiffs, States},
    Resources,
};
use peace_rt_model::{output::OutputWrite, params::ParamsKeys, Error, Flow};

use crate::cmds::{StatesDesiredReadCmd, StatesSavedReadCmd};

#[derive(Debug)]
pub struct DiffCmd<E>(PhantomData<E>);

impl<E> DiffCmd<E>
where
    E: std::error::Error + From<Error> + Send + 'static,
{
    /// Returns the [`state_diff`]`s between the saved current and desired
    /// states.
    ///
    /// Both current and desired states must have been discovered prior to
    /// running this. See [`StatesDiscoverCmd::current_and_desired`].
    ///
    /// [`state_diff`]: peace_cfg::Item::state_diff
    /// [`StatesDiscoverCmd::current_and_desired`]: crate::cmds::StatesDiscoverCmd::current_and_desired
    pub async fn current_and_desired<O, PKeys>(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'_, E, O, PKeys, SetUp>>,
    ) -> Result<StateDiffs, E>
    where
        PKeys: ParamsKeys + 'static,
        O: OutputWrite<E>,
    {
        let states_a = StatesSavedReadCmd::exec(cmd_ctx).await?;
        let states_b = StatesDesiredReadCmd::exec(cmd_ctx).await?;

        let SingleProfileSingleFlowView {
            flow,
            params_specs,
            resources,
            ..
        } = cmd_ctx.view();

        Self::diff_any(flow, params_specs, resources, &states_a, &states_b).await
    }

    /// Returns the [`state_diff`]`s between the saved current states of two
    /// profiles.
    ///
    /// Both profiles' current states must have been discovered prior to
    /// running this. See [`StatesDiscoverCmd::current`].
    ///
    /// [`state_diff`]: peace_cfg::Item::state_diff
    /// [`StatesDiscoverCmd::current`]: crate::cmds::StatesDiscoverCmd::current
    pub async fn diff_profiles_current<O, PKeys>(
        cmd_ctx: &mut CmdCtx<MultiProfileSingleFlow<'_, E, O, PKeys, SetUp>>,
        profile_a: &Profile,
        profile_b: &Profile,
    ) -> Result<StateDiffs, E>
    where
        PKeys: ParamsKeys + 'static,
        O: OutputWrite<E>,
    {
        let MultiProfileSingleFlowView {
            flow,
            profiles,
            profile_to_params_specs,
            profile_to_states_saved,
            resources,
            ..
        } = cmd_ctx.view();

        let params_specs = profile_to_params_specs
            .get(profile_a)
            .or_else(|| profile_to_params_specs.get(profile_b));
        let params_specs = if let Some(Some(params_specs)) = params_specs {
            params_specs
        } else {
            Err(Error::ParamsSpecsNotDefinedForDiff {
                profile_a: profile_a.clone(),
                profile_b: profile_b.clone(),
            })?
        };
        let states_a = profile_to_states_saved
            .get(profile_a)
            .ok_or_else(|| {
                let profile = profile_a.clone();
                let profiles_in_scope = profiles.to_vec();
                Error::ProfileNotInScope {
                    profile,
                    profiles_in_scope,
                }
            })?
            .as_ref()
            .ok_or_else(|| {
                let profile = profile_a.clone();
                Error::ProfileStatesCurrentNotDiscovered { profile }
            })?;
        let states_b = profile_to_states_saved
            .get(profile_b)
            .ok_or_else(|| {
                let profile = profile_b.clone();
                let profiles_in_scope = profiles.to_vec();
                Error::ProfileNotInScope {
                    profile,
                    profiles_in_scope,
                }
            })?
            .as_ref()
            .ok_or_else(|| {
                let profile = profile_b.clone();
                Error::ProfileStatesCurrentNotDiscovered { profile }
            })?;

        Self::diff_any(flow, params_specs, resources, states_a, states_b).await
    }

    /// Returns the [`state_diff`]` for each [`Item`].
    ///
    /// This does not take in `CmdCtx` as it may be used by both
    /// `SingleProfileSingleFlow` and `MultiProfileSingleFlow`
    /// commands.
    ///
    /// [`Item`]: peace_cfg::Item
    /// [`state_diff`]: peace_cfg::Item::state_diff
    pub async fn diff_any<StatesTsA, StatesTsB>(
        flow: &Flow<E>,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        states_a: &States<StatesTsA>,
        states_b: &States<StatesTsB>,
    ) -> Result<StateDiffs, E> {
        let state_diffs = {
            let state_diffs_mut = flow
                .graph()
                .stream()
                .map(Result::<_, E>::Ok)
                .try_filter_map(|item| async move {
                    let state_diff_opt = item
                        .state_diff_exec(params_specs, resources, states_a, states_b)
                        .await?;

                    Ok(state_diff_opt.map(|state_diff| (item.id().clone(), state_diff)))
                })
                .try_collect::<StateDiffsMut>()
                .await?;

            StateDiffs::from(state_diffs_mut)
        };

        Ok(state_diffs)
    }
}

impl<E> Default for DiffCmd<E> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
