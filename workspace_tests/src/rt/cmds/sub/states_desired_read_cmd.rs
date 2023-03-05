use peace::{
    cfg::{app_name, profile, AppName, FlowId, ItemSpec, Profile},
    cmd::ctx::CmdCtx,
    rt::cmds::sub::{StatesDesiredDiscoverCmd, StatesDesiredReadCmd},
    rt_model::{Error, Flow, ItemSpecGraphBuilder, Workspace, WorkspaceSpec},
};

use crate::{NoOpOutput, PeaceTestError, VecCopyError, VecCopyItemSpec, VecCopyState};

#[tokio::test]
async fn reads_states_desired_from_disk_when_present() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemSpecGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItemSpec.into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut output = NoOpOutput;

    // Write desired states to disk.
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .await?;
    let states_desired_from_discover = StatesDesiredDiscoverCmd::exec(&mut cmd_ctx).await?;

    // Re-read states from disk.
    let mut output = NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .await?;
    let states_desired_from_read = StatesDesiredReadCmd::exec(&mut cmd_ctx).await?;

    let vec_copy_state_from_discover =
        states_desired_from_discover.get::<VecCopyState, _>(VecCopyItemSpec.id());
    let vec_copy_state_from_read =
        states_desired_from_read.get::<VecCopyState, _>(VecCopyItemSpec.id());
    assert_eq!(vec_copy_state_from_discover, vec_copy_state_from_read);
    Ok(())
}

#[tokio::test]
async fn returns_error_when_states_not_on_disk() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemSpecGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItemSpec.into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut output = NoOpOutput;

    // Try and read desired states from disk.
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .await?;
    let exec_result = StatesDesiredReadCmd::exec(&mut cmd_ctx).await;

    assert!(matches!(
        exec_result,
        Err(PeaceTestError::PeaceRtError(
            Error::StatesDesiredDiscoverRequired
        ))
    ));
    Ok(())
}

#[test]
fn debug() {
    let debug_str = format!(
        "{:?}",
        StatesDesiredReadCmd::<VecCopyError, NoOpOutput, ()>::default()
    );
    assert!(
        debug_str
            == r#"StatesDesiredReadCmd(PhantomData<(workspace_tests::vec_copy_item_spec::VecCopyError, workspace_tests::no_op_output::NoOpOutput, ())>)"#
            || debug_str == r#"StatesDesiredReadCmd(PhantomData)"#
    );
}
