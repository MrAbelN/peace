use peace::{
    cfg::{profile, state::Nothing, FlowId, ItemSpec, ItemSpecId, Profile, State},
    resources::{
        paths::{StatesCurrentFile, StatesDesiredFile},
        states::{StatesCurrent, StatesDesired},
        type_reg::untagged::{BoxDtDisplay, TypeReg},
    },
    rt::cmds::StatesDiscoverCmd,
    rt_model::{CmdContext, ItemSpecGraphBuilder, Workspace, WorkspaceSpec},
};

use crate::{NoOpOutput, VecCopyError, VecCopyItemSpec, VecCopyState};

#[tokio::test]
async fn runs_state_current_and_state_desired() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
        profile!("test_profile"),
        FlowId::new(crate::fn_name_short!())?,
    )?;
    let graph = {
        let mut graph_builder = ItemSpecGraphBuilder::<VecCopyError>::new();
        graph_builder.add_fn(VecCopyItemSpec.into());
        graph_builder.build()
    };
    let mut no_op_output = NoOpOutput;
    let cmd_context = CmdContext::builder(&workspace, &graph, &mut no_op_output)
        .with_profile_init::<VecCopyState>(Some(VecCopyState::new()))
        .await?;

    let CmdContext { resources, .. } = StatesDiscoverCmd::exec(cmd_context).await?;

    let states = resources.borrow::<StatesCurrent>();
    let states_desired = resources.borrow::<StatesDesired>();
    let vec_copy_state = states.get::<State<VecCopyState, Nothing>, _>(&VecCopyItemSpec.id());
    let states_on_disk = {
        let states_current_file = resources.borrow::<StatesCurrentFile>();
        let states_slice = std::fs::read(&*states_current_file)?;

        let mut type_reg = TypeReg::<ItemSpecId, BoxDtDisplay>::new_typed();
        type_reg.register::<State<VecCopyState, Nothing>>(VecCopyItemSpec.id());

        let deserializer = serde_yaml::Deserializer::from_slice(&states_slice);
        StatesCurrent::from(type_reg.deserialize_map(deserializer)?)
    };
    let vec_copy_desired_state = states_desired.get::<VecCopyState, _>(&VecCopyItemSpec.id());
    let states_desired_on_disk = {
        let states_desired_file = resources.borrow::<StatesDesiredFile>();
        let states_slice = std::fs::read(&*states_desired_file)?;

        let mut type_reg = TypeReg::<ItemSpecId, BoxDtDisplay>::new_typed();
        type_reg.register::<<VecCopyItemSpec as ItemSpec>::StateLogical>(VecCopyItemSpec.id());

        let deserializer = serde_yaml::Deserializer::from_slice(&states_slice);
        StatesDesired::from(type_reg.deserialize_map(deserializer)?)
    };
    assert_eq!(
        Some(State::new(VecCopyState::new(), Nothing)).as_ref(),
        vec_copy_state
    );
    assert_eq!(
        states.get::<State<VecCopyState, Nothing>, _>(&VecCopyItemSpec.id()),
        states_on_disk.get::<State<VecCopyState, Nothing>, _>(&VecCopyItemSpec.id())
    );
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        vec_copy_desired_state
    );
    assert_eq!(
        states_desired.get::<VecCopyState, _>(&VecCopyItemSpec.id()),
        states_desired_on_disk.get::<VecCopyState, _>(&VecCopyItemSpec.id())
    );

    Ok(())
}

#[test]
fn debug() {
    assert_eq!(
        "StatesDiscoverCmd(PhantomData<(workspace_tests::vec_copy_item_spec::VecCopyError, workspace_tests::no_op_output::NoOpOutput)>)",
        format!(
            "{:?}",
            StatesDiscoverCmd::<VecCopyError, NoOpOutput>::default()
        )
    );
}