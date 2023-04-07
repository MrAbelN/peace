use std::{
    fmt::{self, Debug},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use fn_graph::{DataAccess, DataAccessDyn, TypeIds};
use peace_cfg::{async_trait, ItemSpec, ItemSpecId, OpCheckStatus, OpCtx};
use peace_data::{
    marker::{ApplyDry, Clean, Current, Desired},
    Data,
};
use peace_resources::{
    resources::ts::{Empty, SetUp},
    states::{States, StatesCurrent, StatesDesired, StatesSaved},
    type_reg::untagged::BoxDtDisplay,
    Resources,
};

use crate::{
    outcomes::{ItemApply, ItemApplyBoxed, ItemApplyPartial, ItemApplyPartialBoxed},
    ItemSpecRt, StatesTypeReg,
};

/// Wraps a type implementing [`ItemSpec`].
///
/// # Type Parameters
///
/// * `IS`: Item spec type to wrap.
/// * `E`: Application specific error type.
///
///     Notably, `E` here should be the application's error type, which is not
///     necessarily the item spec's error type (unless you have only one item
///     spec in the application).
#[allow(clippy::type_complexity)]
pub struct ItemSpecWrapper<IS, E>(IS, PhantomData<E>);

impl<IS, E> Clone for ItemSpecWrapper<IS, E>
where
    IS: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<IS, E> ItemSpecWrapper<IS, E>
where
    IS: Debug + ItemSpec + Send + Sync,
    E: Debug
        + Send
        + Sync
        + std::error::Error
        + From<<IS as ItemSpec>::Error>
        + From<crate::Error>
        + 'static,
{
    async fn state_clean<ResourcesTs>(
        &self,
        resources: &Resources<ResourcesTs>,
    ) -> Result<IS::State, E> {
        let state_clean = {
            let data =
                <<IS as peace_cfg::ItemSpec>::Data<'_> as Data>::borrow(self.id(), resources);
            <IS as peace_cfg::ItemSpec>::state_clean(data).await?
        };
        resources.borrow_mut::<Clean<IS::State>>().0 = Some(state_clean.clone());

        Ok(state_clean)
    }

    async fn state_current_try_exec<ResourcesTs>(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<ResourcesTs>,
    ) -> Result<Option<IS::State>, E> {
        let state_current = {
            let data =
                <<IS as peace_cfg::ItemSpec>::Data<'_> as Data>::borrow(self.id(), resources);
            <IS as peace_cfg::ItemSpec>::try_state_current(op_ctx, data).await?
        };
        if let Some(state_current) = state_current.as_ref() {
            resources.borrow_mut::<Current<IS::State>>().0 = Some(state_current.clone());
        }

        Ok(state_current)
    }

    async fn state_current_exec<ResourcesTs>(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<ResourcesTs>,
    ) -> Result<IS::State, E> {
        let state_current = {
            let data =
                <<IS as peace_cfg::ItemSpec>::Data<'_> as Data>::borrow(self.id(), resources);
            <IS as peace_cfg::ItemSpec>::state_current(op_ctx, data).await?
        };
        resources.borrow_mut::<Current<IS::State>>().0 = Some(state_current.clone());

        Ok(state_current)
    }

    async fn state_desired_try_exec(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<SetUp>,
    ) -> Result<Option<IS::State>, E> {
        let data = <<IS as peace_cfg::ItemSpec>::Data<'_> as Data>::borrow(self.id(), resources);
        let state_desired = <IS as peace_cfg::ItemSpec>::try_state_desired(op_ctx, data).await?;
        if let Some(state_desired) = state_desired.as_ref() {
            resources.borrow_mut::<Desired<IS::State>>().0 = Some(state_desired.clone());
        }

        Ok(state_desired)
    }

    async fn state_desired_exec(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<SetUp>,
    ) -> Result<IS::State, E> {
        let data = <<IS as peace_cfg::ItemSpec>::Data<'_> as Data>::borrow(self.id(), resources);
        let state_desired = <IS as peace_cfg::ItemSpec>::state_desired(op_ctx, data).await?;
        resources.borrow_mut::<Desired<IS::State>>().0 = Some(state_desired.clone());

        Ok(state_desired)
    }

    async fn state_diff_exec<ResourcesTs, StatesTs>(
        &self,
        resources: &Resources<ResourcesTs>,
        states_base: &States<StatesTs>,
        states_desired: &StatesDesired,
    ) -> Result<Option<IS::StateDiff>, E>
    where
        StatesTs: Debug + Send + Sync + 'static,
    {
        let item_spec_id = <IS as ItemSpec>::id(self);
        let state_base = states_base.get::<IS::State, _>(item_spec_id);
        let state_desired = states_desired.get::<IS::State, _>(item_spec_id);

        if let Some((state_base, state_desired)) = state_base.zip(state_desired) {
            let state_diff: IS::StateDiff = self
                .state_diff_exec_with(resources, state_base, state_desired)
                .await?;
            Ok(Some(state_diff))
        } else {
            // When we reach here, one of the following is true:
            //
            // * The current state cannot be retrieved, due to a predecessor's state not
            //   existing.
            // * The desired state cannot be retrieved, due to a predecessor's state not
            //   existing.
            // * A bug exists, e.g. the state is stored against the wrong type parameter.

            Ok(None)
        }
    }

    async fn state_diff_exec_with<ResourcesTs>(
        &self,
        resources: &Resources<ResourcesTs>,
        state_base: &IS::State,
        state_desired: &IS::State,
    ) -> Result<IS::StateDiff, E> {
        let state_diff: IS::StateDiff = {
            let data =
                <<IS as peace_cfg::ItemSpec>::Data<'_> as Data>::borrow(self.id(), resources);
            <IS as peace_cfg::ItemSpec>::state_diff(data, state_base, state_desired)
                .await
                .map_err(Into::<E>::into)?
        };

        Ok(state_diff)
    }

    async fn apply_op_check<ResourcesTs>(
        &self,
        resources: &Resources<ResourcesTs>,
        state_current: &IS::State,
        state_desired: &IS::State,
        state_diff: &IS::StateDiff,
    ) -> Result<OpCheckStatus, E> {
        let data = <<IS as peace_cfg::ItemSpec>::Data<'_> as Data>::borrow(self.id(), resources);
        <IS as peace_cfg::ItemSpec>::apply_check(data, state_current, state_desired, state_diff)
            .await
            .map_err(Into::<E>::into)
    }

    async fn apply_op_exec_dry<ResourcesTs>(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<ResourcesTs>,
        state_current: &IS::State,
        state_desired: &IS::State,
        state_diff: &IS::StateDiff,
    ) -> Result<IS::State, E> {
        let data = <<IS as peace_cfg::ItemSpec>::Data<'_> as Data>::borrow(self.id(), resources);
        let state_ensured_dry = <IS as peace_cfg::ItemSpec>::apply_dry(
            op_ctx,
            data,
            state_current,
            state_desired,
            state_diff,
        )
        .await
        .map_err(Into::<E>::into)?;

        resources.borrow_mut::<ApplyDry<IS::State>>().0 = Some(state_ensured_dry.clone());

        Ok(state_ensured_dry)
    }

    async fn apply_op_exec<ResourcesTs>(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<ResourcesTs>,
        state_current: &IS::State,
        state_desired: &IS::State,
        state_diff: &IS::StateDiff,
    ) -> Result<IS::State, E> {
        let data = <<IS as peace_cfg::ItemSpec>::Data<'_> as Data>::borrow(self.id(), resources);
        let state_ensured = <IS as peace_cfg::ItemSpec>::apply(
            op_ctx,
            data,
            state_current,
            state_desired,
            state_diff,
        )
        .await
        .map_err(Into::<E>::into)?;

        resources.borrow_mut::<Current<IS::State>>().0 = Some(state_ensured.clone());

        Ok(state_ensured)
    }
}

impl<IS, E> Debug for ItemSpecWrapper<IS, E>
where
    IS: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<IS, E> Deref for ItemSpecWrapper<IS, E> {
    type Target = IS;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<IS, E> DerefMut for ItemSpecWrapper<IS, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<IS, E> From<IS> for ItemSpecWrapper<IS, E>
where
    IS: Debug + ItemSpec + Send + Sync,
    E: Debug + Send + Sync + std::error::Error + From<<IS as ItemSpec>::Error> + 'static,
{
    fn from(item_spec: IS) -> Self {
        Self(item_spec, PhantomData)
    }
}

impl<IS, E> DataAccess for ItemSpecWrapper<IS, E>
where
    IS: Debug + ItemSpec + Send + Sync,
    E: Debug + Send + Sync + std::error::Error + From<<IS as ItemSpec>::Error> + 'static,
{
    fn borrows() -> TypeIds {
        <<IS as peace_cfg::ItemSpec>::Data<'_> as DataAccess>::borrows()
    }

    fn borrow_muts() -> TypeIds {
        <<IS as peace_cfg::ItemSpec>::Data<'_> as DataAccess>::borrow_muts()
    }
}

impl<IS, E> DataAccessDyn for ItemSpecWrapper<IS, E>
where
    IS: Debug + ItemSpec + Send + Sync,
    E: Debug + Send + Sync + std::error::Error + From<<IS as ItemSpec>::Error> + 'static,
{
    fn borrows(&self) -> TypeIds {
        <<IS as peace_cfg::ItemSpec>::Data<'_> as DataAccess>::borrows()
    }

    fn borrow_muts(&self) -> TypeIds {
        <<IS as peace_cfg::ItemSpec>::Data<'_> as DataAccess>::borrow_muts()
    }
}

#[async_trait(?Send)]
impl<IS, E> ItemSpecRt<E> for ItemSpecWrapper<IS, E>
where
    IS: Clone + Debug + ItemSpec + Send + Sync,
    E: Debug
        + Send
        + Sync
        + std::error::Error
        + From<<IS as ItemSpec>::Error>
        + From<crate::Error>
        + 'static,
{
    fn id(&self) -> &ItemSpecId {
        <IS as ItemSpec>::id(self)
    }

    async fn setup(&self, resources: &mut Resources<Empty>) -> Result<(), E> {
        // Insert `XMarker<IS::State>` to create entries in `Resources`.
        // This is used for referential param values (#94)
        resources.insert(Clean::<IS::State>(None));
        resources.insert(Current::<IS::State>(None));
        resources.insert(Desired::<IS::State>(None));
        resources.insert(ApplyDry::<IS::State>(None));

        // Run user defined setup.
        <IS as ItemSpec>::setup(self, resources)
            .await
            .map_err(Into::<E>::into)
    }

    fn state_register(&self, states_type_reg: &mut StatesTypeReg) {
        states_type_reg.register::<IS::State>(<IS as ItemSpec>::id(self).clone());
    }

    async fn state_clean(&self, resources: &Resources<SetUp>) -> Result<BoxDtDisplay, E> {
        self.state_clean(resources)
            .await
            .map(BoxDtDisplay::new)
            .map_err(Into::<E>::into)
    }

    async fn state_current_try_exec(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<SetUp>,
    ) -> Result<Option<BoxDtDisplay>, E> {
        self.state_current_try_exec(op_ctx, resources)
            .await
            .map(|state_current| state_current.map(BoxDtDisplay::new))
            .map_err(Into::<E>::into)
    }

    async fn state_current_exec(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<SetUp>,
    ) -> Result<BoxDtDisplay, E> {
        self.state_current_exec(op_ctx, resources)
            .await
            .map(BoxDtDisplay::new)
            .map_err(Into::<E>::into)
    }

    async fn state_desired_try_exec(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<SetUp>,
    ) -> Result<Option<BoxDtDisplay>, E> {
        self.state_desired_try_exec(op_ctx, resources)
            .await
            .map(|state_desired| state_desired.map(BoxDtDisplay::new))
            .map_err(Into::<E>::into)
    }

    async fn state_desired_exec(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<SetUp>,
    ) -> Result<BoxDtDisplay, E> {
        self.state_desired_exec(op_ctx, resources)
            .await
            .map(BoxDtDisplay::new)
            .map_err(Into::<E>::into)
    }

    async fn state_diff_exec_with_states_saved(
        &self,
        resources: &Resources<SetUp>,
        states_saved: &StatesSaved,
        states_desired: &StatesDesired,
    ) -> Result<Option<BoxDtDisplay>, E> {
        self.state_diff_exec(resources, states_saved, states_desired)
            .await
            .map(|state_diff_opt| state_diff_opt.map(BoxDtDisplay::new))
            .map_err(Into::<E>::into)
    }

    async fn state_diff_exec_with_states_current(
        &self,
        resources: &Resources<SetUp>,
        states_current: &StatesCurrent,
        states_desired: &StatesDesired,
    ) -> Result<Option<BoxDtDisplay>, E> {
        self.state_diff_exec(resources, states_current, states_desired)
            .await
            .map(|state_diff_opt| state_diff_opt.map(BoxDtDisplay::new))
            .map_err(Into::<E>::into)
    }

    async fn ensure_prepare(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<SetUp>,
    ) -> Result<ItemApplyBoxed, (E, ItemApplyPartialBoxed)> {
        let mut item_apply_partial = ItemApplyPartial::<IS::State, IS::StateDiff>::new();

        match self.state_current_exec(op_ctx, resources).await {
            Ok(state_current) => item_apply_partial.state_current = Some(state_current),
            Err(error) => return Err((error, item_apply_partial.into())),
        }
        #[cfg(feature = "output_progress")]
        op_ctx.progress_sender().reset();
        match self.state_desired_exec(op_ctx, resources).await {
            Ok(state_desired) => item_apply_partial.state_target = Some(state_desired),
            Err(error) => return Err((error, item_apply_partial.into())),
        }
        #[cfg(feature = "output_progress")]
        op_ctx.progress_sender().reset();
        match self
            .state_diff_exec_with(
                resources,
                item_apply_partial
                    .state_current
                    .as_ref()
                    .expect("unreachable: This is set just above."),
                item_apply_partial
                    .state_target
                    .as_ref()
                    .expect("unreachable: This is set just above."),
            )
            .await
        {
            Ok(state_diff) => item_apply_partial.state_diff = Some(state_diff),
            Err(error) => return Err((error, item_apply_partial.into())),
        }

        let (Some(state_current), Some(state_target), Some(state_diff)) = (
            item_apply_partial.state_current.as_ref(),
            item_apply_partial.state_target.as_ref(),
            item_apply_partial.state_diff.as_ref(),
        ) else {
            unreachable!("These are set just above.");
        };

        let state_applied = match self
            .apply_op_check(resources, state_current, state_target, state_diff)
            .await
        {
            Ok(op_check_status) => {
                item_apply_partial.op_check_status = Some(op_check_status);

                // TODO: write test for this case
                match op_check_status {
                    #[cfg(not(feature = "output_progress"))]
                    OpCheckStatus::ExecRequired => None,
                    #[cfg(feature = "output_progress")]
                    OpCheckStatus::ExecRequired { .. } => None,
                    OpCheckStatus::ExecNotRequired => item_apply_partial.state_current.clone(),
                }
            }
            Err(error) => return Err((error, item_apply_partial.into())),
        };

        Ok(ItemApply::try_from((item_apply_partial, state_applied))
            .expect("unreachable: All the fields are set above.")
            .into())
    }

    async fn apply_exec_dry(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<SetUp>,
        item_apply_boxed: &mut ItemApplyBoxed,
    ) -> Result<(), E> {
        let Some(item_apply) =
            item_apply_boxed.as_data_type_mut().downcast_mut::<ItemApply<IS::State, IS::StateDiff>>() else {
                panic!("Failed to downcast `ItemApplyBoxed` to `{concrete_type}`.\n\
                    This is a bug in the Peace framework.",
                    concrete_type = std::any::type_name::<ItemApply<IS::State, IS::StateDiff>>())
            };

        let ItemApply {
            state_saved: _,
            state_current,
            state_target,
            state_diff,
            op_check_status,
            state_applied,
        } = item_apply;

        match op_check_status {
            #[cfg(not(feature = "output_progress"))]
            OpCheckStatus::ExecRequired => {
                let state_applied_dry = self
                    .apply_op_exec_dry(op_ctx, resources, state_current, state_target, state_diff)
                    .await?;

                *state_applied = Some(state_applied_dry);
            }
            #[cfg(feature = "output_progress")]
            OpCheckStatus::ExecRequired { progress_limit: _ } => {
                let state_applied_dry = self
                    .apply_op_exec_dry(op_ctx, resources, state_current, state_target, state_diff)
                    .await?;

                *state_applied = Some(state_applied_dry);
            }
            OpCheckStatus::ExecNotRequired => {}
        }

        Ok(())
    }

    async fn clean_prepare(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<SetUp>,
    ) -> Result<ItemApplyBoxed, (E, ItemApplyPartialBoxed)> {
        let mut item_apply_partial = ItemApplyPartial::<IS::State, IS::StateDiff>::new();

        match self.state_current_try_exec(op_ctx, resources).await {
            Ok(state_current) => {
                // Hack: Setting ItemApplyPartial state_current to state_clean is a hack.
                if let Some(state_current) = state_current {
                    item_apply_partial.state_current = Some(state_current);
                } else {
                    match self.state_clean(resources).await {
                        Ok(state_clean) => item_apply_partial.state_current = Some(state_clean),
                        Err(error) => return Err((error, item_apply_partial.into())),
                    }
                }
            }
            Err(error) => return Err((error, item_apply_partial.into())),
        }
        match self.state_clean(resources).await {
            Ok(state_clean) => item_apply_partial.state_target = Some(state_clean),
            Err(error) => return Err((error, item_apply_partial.into())),
        }

        match self
            .state_diff_exec_with(
                resources,
                item_apply_partial
                    .state_current
                    .as_ref()
                    .expect("unreachable: This is confirmed just above."),
                item_apply_partial
                    .state_target
                    .as_ref()
                    .expect("unreachable: This is set just above."),
            )
            .await
        {
            Ok(state_diff) => item_apply_partial.state_diff = Some(state_diff),
            Err(error) => return Err((error, item_apply_partial.into())),
        }

        let (Some(state_current), Some(state_target), Some(state_diff)) = (
            item_apply_partial.state_current.as_ref(),
            item_apply_partial.state_target.as_ref(),
            item_apply_partial.state_diff.as_ref(),
        ) else {
            unreachable!("These are set just above.");
        };

        let state_applied = match self
            .apply_op_check(resources, state_current, state_target, state_diff)
            .await
        {
            Ok(op_check_status) => {
                item_apply_partial.op_check_status = Some(op_check_status);

                // TODO: write test for this case
                match op_check_status {
                    #[cfg(not(feature = "output_progress"))]
                    OpCheckStatus::ExecRequired => None,
                    #[cfg(feature = "output_progress")]
                    OpCheckStatus::ExecRequired { .. } => None,
                    OpCheckStatus::ExecNotRequired => item_apply_partial.state_current.clone(),
                }
            }
            Err(error) => return Err((error, item_apply_partial.into())),
        };

        Ok(ItemApply::try_from((item_apply_partial, state_applied))
            .expect("unreachable: All the fields are set above.")
            .into())
    }

    async fn apply_exec(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<SetUp>,
        item_apply_boxed: &mut ItemApplyBoxed,
    ) -> Result<(), E> {
        let Some(item_apply) =
            item_apply_boxed.as_data_type_mut().downcast_mut::<ItemApply<IS::State, IS::StateDiff>>() else {
                panic!("Failed to downcast `ItemApplyBoxed` to `{concrete_type}`.\n\
                    This is a bug in the Peace framework.",
                    concrete_type = std::any::type_name::<ItemApply<IS::State, IS::StateDiff>>())
            };

        let ItemApply {
            state_saved: _,
            state_current,
            state_target,
            state_diff,
            op_check_status,
            state_applied,
        } = item_apply;

        match op_check_status {
            #[cfg(not(feature = "output_progress"))]
            OpCheckStatus::ExecRequired => {
                let state_applied_next = self
                    .apply_op_exec(op_ctx, resources, state_current, state_target, state_diff)
                    .await?;

                *state_applied = Some(state_applied_next);
            }
            #[cfg(feature = "output_progress")]
            OpCheckStatus::ExecRequired { progress_limit: _ } => {
                let state_applied_next = self
                    .apply_op_exec(op_ctx, resources, state_current, state_target, state_diff)
                    .await?;

                *state_applied = Some(state_applied_next);
            }
            OpCheckStatus::ExecNotRequired => {}
        }

        Ok(())
    }
}
