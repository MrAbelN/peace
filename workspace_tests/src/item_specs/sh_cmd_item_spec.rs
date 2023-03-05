use peace::{
    cfg::{app_name, item_spec_id, profile, AppName, FlowId, ItemSpecId, Profile, State},
    cmd::ctx::CmdCtx,
    resources::states::StatesSaved,
    rt::cmds::{
        sub::{StatesCurrentDiscoverCmd, StatesDesiredDiscoverCmd, StatesSavedReadCmd},
        CleanCmd, DiffCmd, EnsureCmd, StatesDiscoverCmd,
    },
    rt_model::{Flow, InMemoryTextOutput, ItemSpecGraphBuilder, Workspace, WorkspaceSpec},
};
use peace_item_specs::sh_cmd::{
    ShCmd, ShCmdError, ShCmdExecutionRecord, ShCmdItemSpec, ShCmdParams, ShCmdState, ShCmdStateDiff,
};

/// Creates a file.
#[derive(Clone, Copy, Debug)]
pub struct TestFileCreationShCmdItemSpec;

pub type TestFileCreationShCmdStateLogical = ShCmdState<TestFileCreationShCmdItemSpec>;
pub type TestFileCreationShCmdState =
    State<TestFileCreationShCmdStateLogical, ShCmdExecutionRecord>;

impl TestFileCreationShCmdItemSpec {
    /// ID
    pub const ID: ItemSpecId = item_spec_id!("test_file_creation");

    /// Returns a new `TestFileCreationShCmdItemSpec`.
    pub fn new() -> ShCmdItemSpec<Self> {
        #[cfg(unix)]
        let sh_cmd_params = {
            let state_current_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item_spec/unix/test_file_creation_state_current.sh"
            ));

            let state_desired_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item_spec/unix/test_file_creation_state_desired.sh"
            ));
            let state_diff_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item_spec/unix/test_file_creation_state_diff.sh"
            ));
            let ensure_check_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item_spec/unix/test_file_creation_ensure_check.sh"
            ));
            let ensure_exec_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item_spec/unix/test_file_creation_ensure_exec.sh"
            ));
            let clean_check_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item_spec/unix/test_file_creation_clean_check.sh"
            ));
            let clean_exec_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item_spec/unix/test_file_creation_clean_exec.sh"
            ));
            ShCmdParams::<TestFileCreationShCmdItemSpec>::new(
                state_current_sh_cmd,
                state_desired_sh_cmd,
                state_diff_sh_cmd,
                ensure_check_sh_cmd,
                ensure_exec_sh_cmd,
                clean_check_sh_cmd,
                clean_exec_sh_cmd,
            )
        };

        #[cfg(windows)]
        let sh_cmd_params = {
            let state_current_sh_cmd =
                ShCmd::new("Powershell.exe")
                    .arg("-Command")
                    .arg(include_str!(
                        "sh_cmd_item_spec/windows/test_file_creation_state_current.ps1"
                    ));

            let state_desired_sh_cmd =
                ShCmd::new("Powershell.exe")
                    .arg("-Command")
                    .arg(include_str!(
                        "sh_cmd_item_spec/windows/test_file_creation_state_desired.ps1"
                    ));
            let state_diff_sh_cmd = ShCmd::new("Powershell.exe").arg("-Command").arg(concat!(
                "& { ",
                include_str!("sh_cmd_item_spec/windows/test_file_creation_state_diff.ps1"),
                " }"
            ));
            let ensure_check_sh_cmd = ShCmd::new("Powershell.exe").arg("-Command").arg(concat!(
                "& { ",
                include_str!("sh_cmd_item_spec/windows/test_file_creation_ensure_check.ps1"),
                " }"
            ));
            let ensure_exec_sh_cmd = ShCmd::new("Powershell.exe").arg("-Command").arg(concat!(
                "& { ",
                include_str!("sh_cmd_item_spec/windows/test_file_creation_ensure_exec.ps1"),
                " }"
            ));
            let clean_check_sh_cmd = ShCmd::new("Powershell.exe").arg("-Command").arg(concat!(
                "& { ",
                include_str!("sh_cmd_item_spec/windows/test_file_creation_clean_check.ps1"),
                " }"
            ));
            let clean_exec_sh_cmd = ShCmd::new("Powershell.exe").arg("-Command").arg(concat!(
                "& { ",
                include_str!("sh_cmd_item_spec/windows/test_file_creation_clean_exec.ps1"),
                " }"
            ));
            ShCmdParams::<TestFileCreationShCmdItemSpec>::new(
                state_current_sh_cmd,
                state_desired_sh_cmd,
                state_diff_sh_cmd,
                ensure_check_sh_cmd,
                ensure_exec_sh_cmd,
                clean_check_sh_cmd,
                clean_exec_sh_cmd,
            )
        };

        ShCmdItemSpec::new(Self::ID, Some(sh_cmd_params))
    }
}

#[tokio::test]
async fn state_current_returns_shell_command_current_state()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemSpecGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItemSpec::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .await?;

    let states_current = StatesCurrentDiscoverCmd::exec(&mut cmd_ctx).await?;
    let state_current = states_current
        .get::<TestFileCreationShCmdState, _>(&TestFileCreationShCmdItemSpec::ID)
        .unwrap();
    if let ShCmdState::Some {
        stdout,
        stderr,
        marker: _,
    } = &state_current.logical
    {
        assert_eq!("not_exists", stdout);
        assert_eq!("`test_file` does not exist", stderr);
    } else {
        panic!(
            "Expected `state_current` to be `ShCmdState::Some` after `StatesCurrent` discovery."
        );
    }

    Ok(())
}

#[tokio::test]
async fn state_desired_returns_shell_command_desired_state()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemSpecGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItemSpec::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .await?;

    let states_desired = StatesDesiredDiscoverCmd::exec(&mut cmd_ctx).await?;
    let state_desired = states_desired
        .get::<State<TestFileCreationShCmdStateLogical, ShCmdExecutionRecord>, _>(
            &TestFileCreationShCmdItemSpec::ID,
        )
        .unwrap();
    if let ShCmdState::Some {
        stdout,
        stderr,
        marker: _,
    } = &state_desired.logical
    {
        assert_eq!("exists", stdout);
        assert_eq!("`test_file` exists", stderr);
    } else {
        panic!(
            "Expected `state_desired` to be `ShCmdState::Some` after `StatesDesired` discovery."
        );
    }

    Ok(())
}

#[tokio::test]
async fn state_diff_returns_shell_command_state_diff() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemSpecGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItemSpec::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .await?;

    // Discover states current and desired
    StatesDiscoverCmd::exec(&mut cmd_ctx).await?;

    // Diff them
    let state_diffs = DiffCmd::exec(&mut cmd_ctx).await?;

    let state_diff = state_diffs
        .get::<ShCmdStateDiff, _>(&TestFileCreationShCmdItemSpec::ID)
        .unwrap();
    assert_eq!("creation_required", state_diff.stdout());
    assert_eq!("`test_file` will be created", state_diff.stderr());

    Ok(())
}

#[tokio::test]
async fn ensure_when_creation_required_executes_ensure_exec_shell_command()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemSpecGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItemSpec::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .await?;

    // Discover states current and desired
    let (states_current, _states_desired) = StatesDiscoverCmd::exec(&mut cmd_ctx).await?;
    let states_saved = StatesSaved::from(states_current);

    // Create the file
    let states_ensured = EnsureCmd::exec(&mut cmd_ctx, &states_saved).await?;

    let state_ensured = states_ensured
        .get::<TestFileCreationShCmdState, _>(&TestFileCreationShCmdItemSpec::ID)
        .unwrap();
    if let ShCmdState::Some {
        stdout,
        stderr,
        marker: _,
    } = &state_ensured.logical
    {
        assert_eq!("exists", stdout);
        assert_eq!("`test_file` exists", stderr);
    } else {
        panic!("Expected `state_ensured` to be `ShCmdState::Some` after `EnsureCmd` execution.");
    }

    Ok(())
}

#[tokio::test]
async fn ensure_when_exists_sync_does_not_reexecute_ensure_exec_shell_command()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemSpecGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItemSpec::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .await?;

    // Discover states current and desired
    let (states_current, _states_desired) = StatesDiscoverCmd::exec(&mut cmd_ctx).await?;
    let states_saved = StatesSaved::from(states_current);

    // Create the file
    EnsureCmd::exec(&mut cmd_ctx, &states_saved).await?;

    // Diff state after creation
    let state_diffs = DiffCmd::exec(&mut cmd_ctx).await?;

    let state_diff = state_diffs
        .get::<ShCmdStateDiff, _>(&TestFileCreationShCmdItemSpec::ID)
        .unwrap();
    assert_eq!("exists_sync", state_diff.stdout());
    assert_eq!("nothing to do", state_diff.stderr());

    // Run again, for idempotence checck
    let states_saved = StatesSavedReadCmd::exec(&mut cmd_ctx).await?;
    let states_ensured = EnsureCmd::exec(&mut cmd_ctx, &states_saved).await?;

    let state_ensured = states_ensured
        .get::<TestFileCreationShCmdState, _>(&TestFileCreationShCmdItemSpec::ID)
        .unwrap();
    if let ShCmdState::Some {
        stdout,
        stderr,
        marker: _,
    } = &state_ensured.logical
    {
        assert_eq!("exists", stdout);
        assert_eq!("`test_file` exists", stderr);
    } else {
        panic!("Expected `state_ensured` to be `ShCmdState::Some` after `EnsureCmd` execution.");
    }

    Ok(())
}

#[tokio::test]
async fn clean_when_exists_sync_executes_shell_command() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemSpecGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItemSpec::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .await?;

    // Discover states current and desired
    let (states_current, _states_desired) = StatesDiscoverCmd::exec(&mut cmd_ctx).await?;
    let states_saved = StatesSaved::from(states_current);

    // Create the file
    EnsureCmd::exec(&mut cmd_ctx, &states_saved).await?;

    assert!(tempdir.path().join("test_file").exists());

    // Clean the file
    CleanCmd::exec(&mut cmd_ctx).await?;

    assert!(!tempdir.path().join("test_file").exists());

    // Run again, for idempotence checck
    let states_cleaned = CleanCmd::exec(&mut cmd_ctx).await?;

    let state_cleaned = states_cleaned
        .get::<TestFileCreationShCmdState, _>(&TestFileCreationShCmdItemSpec::ID)
        .unwrap();
    if let ShCmdState::Some {
        stdout,
        stderr,
        marker: _,
    } = &state_cleaned.logical
    {
        assert_eq!("not_exists", stdout);
        assert_eq!("`test_file` does not exist", stderr);
    } else {
        panic!("Expected `state_cleaned` to be `ShCmdState::Some` after `CleanCmd` execution.");
    }

    Ok(())
}
