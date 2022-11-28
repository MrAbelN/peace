use std::marker::PhantomData;

use crate::states::{
    ts::{Current, Previous},
    States,
};

/// Previous `State`s for all `ItemSpec`s.
///
/// This is loaded into [`Resources`] at the beginning of any command execution,
/// from the [`StatesCurrentFile`].
///
/// This is distinct from [`StatesCurrent`] to address the following use cases:
///
/// * Discovering current state from what is recorded in the
///   [`StatesCurrentFile`].
/// * Discovering current state and comparing it with previous state within the
///   same execution.
///
/// # Implementors
///
/// If an `ItemSpec`'s state discovery depends on the `State` of a previous
/// `ItemSpec`, then you should insert the predecessor's state into
/// [`Resources`], and reference that in the subsequent `FnSpec`'s [`Data`]:
///
/// ```rust
/// # use std::path::PathBuf;
/// #
/// # use peace_data::{Data, R};
/// #
/// /// Predecessor `FnSpec::Data`.
/// #[derive(Data, Debug)]
/// pub struct AppUploadParams<'op> {
///     /// Path to the application directory.
///     app_dir: W<'op, PathBuf>,
/// }
///
/// /// Successor `FnSpec::Data`.
/// #[derive(Data, Debug)]
/// pub struct AppInstallParams<'op> {
///     /// Path to the application directory.
///     app_dir: R<'op, PathBuf>,
///     /// Configuration to use.
///     config: W<'op, String>,
/// }
/// ```
///
/// You may reference [`StatesPrevious`] in `EnsureOpSpec::Data` for reading. It
/// is not mutable as `StatesPrevious` must remain unchanged so that all
/// `ItemSpec`s operate over consistent data.
///
/// [`StatesCurrentFile`]: crate::paths::StatesCurrentFile
/// [`Data`]: peace_data::Data
/// [`Resources`]: crate::Resources
pub type StatesPrevious = States<Previous>;

impl From<States<Current>> for States<Previous> {
    fn from(states_current: States<Current>) -> Self {
        let States(type_map, PhantomData) = states_current;

        Self(type_map, PhantomData)
    }
}