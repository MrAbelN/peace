//! Contains type-erased `ItemSpec` types and traits.
//!
//! Types and traits in this module don't reference any associated types from
//! the `ItemSpec`, allowing them to be passed around as common types at compile
//! time.
//!
//! For the logic that is aware of the type parameters, see the
//! [`item_spec_wrapper`] module and [`ItemSpecWrapper`] type.
//!
//! [`item_spec_wrapper`]: crate::item_spec_wrapper
//! [`ItemSpecWrapper`]: crate::ItemSpecWrapper

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use fn_graph::{DataAccessDyn, TypeIds};
use peace_cfg::{FnSpec, ItemSpec, State};
use serde::{de::DeserializeOwned, Serialize};

use crate::{ItemSpecRt, ItemSpecWrapper};

/// Holds a type-erased `ItemSpecWrapper` in a `Box`.
///
/// # Type Parameters
///
/// * `E`: Application specific error type.
#[derive(Debug)]
pub struct ItemSpecBoxed<E>(Box<dyn ItemSpecRt<E>>)
where
    E: std::error::Error;

impl<E> Deref for ItemSpecBoxed<E>
where
    E: std::error::Error,
{
    type Target = dyn ItemSpecRt<E>;

    // https://github.com/rust-lang/rust-clippy/issues/9101
    #[allow(clippy::explicit_auto_deref)]
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<E> DerefMut for ItemSpecBoxed<E>
where
    E: std::error::Error,
{
    // https://github.com/rust-lang/rust-clippy/issues/9101
    #[allow(clippy::explicit_auto_deref)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

impl<
    IS,
    E,
    StateLogical,
    StatePhysical,
    StateDiff,
    StateCurrentFnSpec,
    StateDesiredFnSpec,
    StateDiffFnSpec,
    EnsureOpSpec,
    CleanOpSpec,
> From<IS> for ItemSpecBoxed<E>
where
    IS: Debug
        + ItemSpec<
            Error = E,
            StateLogical = StateLogical,
            StatePhysical = StatePhysical,
            StateDiff = StateDiff,
            StateCurrentFnSpec = StateCurrentFnSpec,
            StateDesiredFnSpec = StateDesiredFnSpec,
            StateDiffFnSpec = StateDiffFnSpec,
            EnsureOpSpec = EnsureOpSpec,
            CleanOpSpec = CleanOpSpec,
        > + Send
        + Sync
        + 'static,
    E: Debug + Send + Sync + std::error::Error + 'static,
    StateLogical: Clone + Debug + Serialize + DeserializeOwned + Send + Sync + 'static,
    StatePhysical: Clone + Debug + Serialize + DeserializeOwned + Send + Sync + 'static,
    StateDiff: Clone + Debug + Serialize + DeserializeOwned + Send + Sync + 'static,
    StateCurrentFnSpec: Debug
        + FnSpec<Error = E, Output = State<StateLogical, StatePhysical>>
        + Send
        + Sync
        + 'static,
    StateDesiredFnSpec: Debug + FnSpec<Error = E, Output = StateLogical> + Send + Sync + 'static,
    StateDiffFnSpec: Debug
        + peace_cfg::StateDiffFnSpec<
            Error = E,
            StateLogical = StateLogical,
            StatePhysical = StatePhysical,
            StateDiff = StateDiff,
        > + Send
        + Sync
        + 'static,
    EnsureOpSpec: Debug
        + peace_cfg::EnsureOpSpec<
            Error = E,
            StateLogical = StateLogical,
            StatePhysical = StatePhysical,
            StateDiff = StateDiff,
        > + Send
        + Sync
        + 'static,
    CleanOpSpec: Debug
        + peace_cfg::CleanOpSpec<
            Error = E,
            StateLogical = StateLogical,
            StatePhysical = StatePhysical,
        > + Send
        + Sync
        + 'static,
{
    fn from(item_spec: IS) -> Self {
        Self(Box::new(ItemSpecWrapper::from(item_spec)))
    }
}

impl<E> DataAccessDyn for ItemSpecBoxed<E>
where
    E: std::error::Error,
{
    fn borrows(&self) -> TypeIds {
        DataAccessDyn::borrows(self.0.as_ref())
    }

    fn borrow_muts(&self) -> TypeIds {
        DataAccessDyn::borrow_muts(self.0.as_ref())
    }
}