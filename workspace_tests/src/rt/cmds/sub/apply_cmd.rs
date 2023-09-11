use peace::{
    resources::states::ts::{Current, CurrentStored},
    rt::{cmd_blocks::apply_exec_cmd_block::ApplyFor, cmds::sub::ApplyCmd},
};

use crate::{NoOpOutput, VecCopyError};

mod apply_stored_state_sync;

#[test]
fn clone() {
    assert_eq!(ApplyFor::Ensure, Clone::clone(&ApplyFor::Ensure));
    assert_ne!(ApplyFor::Ensure, ApplyFor::Clean);
}

#[test]
fn debug() {
    let debug_str = format!(
        "{:?}",
        ApplyCmd::<VecCopyError, NoOpOutput, (), Current, CurrentStored>::default()
    );
    assert_eq!(
        "ApplyCmd(PhantomData<(\
            workspace_tests::vec_copy_item::VecCopyError, \
            workspace_tests::no_op_output::NoOpOutput, \
            (), \
            peace_resources::states::ts::Current, \
            peace_resources::states::ts::CurrentStored\
        )>)",
        debug_str,
    );
}
