use std::marker::PhantomData;

use crate::{
    resources_type_state::WithStateDiffs,
    states::{ts::Ensured, States},
    Resources, StatesCurrent,
};

/// Ensured `State`s for all `ItemSpec`s. `TypeMap<ItemSpecId>` newtype.
///
/// These are the `State`s collected after `EnsureOpSpec::exec` has been run.
///
/// # Implementors
///
/// You may reference [`StatesEnsured`] after `EnsureCmd::exec` has been run.
///
/// [`Data`]: peace_data::Data
pub type StatesEnsured = States<Ensured>;

/// `Resources` is not used at runtime, but is present to signal this type
/// should only be constructed by `EnsureCmd`.
impl From<(StatesCurrent, &Resources<WithStateDiffs>)> for StatesEnsured {
    fn from((states, _resources): (StatesCurrent, &Resources<WithStateDiffs>)) -> Self {
        Self(states.into_inner(), PhantomData)
    }
}
