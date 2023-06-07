use peace::{
    cfg::{app_name, item_id, profile, AppName, FlowId, ItemId, Profile, State},
    cmd::{ctx::CmdCtx, scopes::SingleProfileSingleFlowView},
    data::marker::Clean,
    resources::states::StatesSaved,
    rt::cmds::{CleanCmd, DiffCmd, EnsureCmd, StatesDiscoverCmd, StatesSavedReadCmd},
    rt_model::{
        outcomes::CmdOutcome, Flow, InMemoryTextOutput, ItemGraphBuilder, Workspace, WorkspaceSpec,
    },
};
use peace_items::sh_cmd::{
    ShCmd, ShCmdError, ShCmdExecutionRecord, ShCmdItem, ShCmdParams, ShCmdState, ShCmdStateDiff,
};

/// Creates a file.
#[derive(Clone, Copy, Debug)]
pub struct TestFileCreationShCmdItem;

pub type TestFileCreationShCmdStateLogical = ShCmdState<TestFileCreationShCmdItem>;
pub type TestFileCreationShCmdState =
    State<TestFileCreationShCmdStateLogical, ShCmdExecutionRecord>;

impl TestFileCreationShCmdItem {
    /// ID
    pub const ID: ItemId = item_id!("test_file_creation");

    /// Returns a new `TestFileCreationShCmdItem`.
    pub fn new() -> ShCmdItem<Self> {
        ShCmdItem::new(Self::ID)
    }

    fn params() -> ShCmdParams<TestFileCreationShCmdItem> {
        #[cfg(unix)]
        let sh_cmd_params = {
            let state_clean_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item/unix/test_file_creation_state_clean.sh"
            ));
            let state_current_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item/unix/test_file_creation_state_current.sh"
            ));
            let state_desired_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item/unix/test_file_creation_state_desired.sh"
            ));
            let state_diff_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item/unix/test_file_creation_state_diff.sh"
            ));
            let apply_check_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item/unix/test_file_creation_apply_check.sh"
            ));
            let apply_exec_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item/unix/test_file_creation_apply_exec.sh"
            ));
            ShCmdParams::<TestFileCreationShCmdItem>::new(
                state_clean_sh_cmd,
                state_current_sh_cmd,
                state_desired_sh_cmd,
                state_diff_sh_cmd,
                apply_check_sh_cmd,
                apply_exec_sh_cmd,
            )
        };

        #[cfg(windows)]
        let sh_cmd_params = {
            let state_clean_sh_cmd =
                ShCmd::new("Powershell.exe")
                    .arg("-Command")
                    .arg(include_str!(
                        "sh_cmd_item/windows/test_file_creation_state_clean.ps1"
                    ));
            let state_current_sh_cmd =
                ShCmd::new("Powershell.exe")
                    .arg("-Command")
                    .arg(include_str!(
                        "sh_cmd_item/windows/test_file_creation_state_current.ps1"
                    ));
            let state_desired_sh_cmd =
                ShCmd::new("Powershell.exe")
                    .arg("-Command")
                    .arg(include_str!(
                        "sh_cmd_item/windows/test_file_creation_state_desired.ps1"
                    ));
            let state_diff_sh_cmd = ShCmd::new("Powershell.exe").arg("-Command").arg(concat!(
                "& { ",
                include_str!("sh_cmd_item/windows/test_file_creation_state_diff.ps1"),
                " }"
            ));
            let apply_check_sh_cmd = ShCmd::new("Powershell.exe").arg("-Command").arg(concat!(
                "& { ",
                include_str!("sh_cmd_item/windows/test_file_creation_apply_check.ps1"),
                " }"
            ));
            let apply_exec_sh_cmd = ShCmd::new("Powershell.exe").arg("-Command").arg(concat!(
                "& { ",
                include_str!("sh_cmd_item/windows/test_file_creation_apply_exec.ps1"),
                " }"
            ));
            ShCmdParams::<TestFileCreationShCmdItem>::new(
                state_clean_sh_cmd,
                state_current_sh_cmd,
                state_desired_sh_cmd,
                state_diff_sh_cmd,
                apply_check_sh_cmd,
                apply_exec_sh_cmd,
            )
        };

        sh_cmd_params
    }
}

#[tokio::test]
async fn state_clean_returns_shell_command_clean_state() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItem::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_params::<ShCmdItem<TestFileCreationShCmdItem>>(
            TestFileCreationShCmdItem::ID,
            TestFileCreationShCmdItem::params().into(),
        )
        .await?;

    let CmdOutcome {
        value: (states_current, _states_desired),
        errors: _,
    } = StatesDiscoverCmd::current_and_desired(&mut cmd_ctx).await?;
    let states_saved = StatesSaved::from(states_current);
    CleanCmd::exec_dry(&mut cmd_ctx, &states_saved).await?;
    let state_clean = cmd_ctx
        .resources()
        .borrow::<Clean<TestFileCreationShCmdState>>();
    let Some(state_clean) = state_clean
        .as_ref() else {
            panic!("Expected `Clean<TestFileCreationShCmdState>` to be Some after `CleanCmd::exec_dry`.");
        };
    if let ShCmdState::Some {
        stdout,
        stderr,
        marker: _,
    } = &state_clean.logical
    {
        assert_eq!("not_exists", stdout);
        assert_eq!("`test_file` does not exist", stderr);
    } else {
        panic!("Expected `state_clean` to be `ShCmdState::Some` after `CleanCmd::exec_dry`.");
    }

    Ok(())
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
        let mut graph_builder = ItemGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItem::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_params::<ShCmdItem<TestFileCreationShCmdItem>>(
            TestFileCreationShCmdItem::ID,
            TestFileCreationShCmdItem::params().into(),
        )
        .await?;

    let CmdOutcome {
        value: states_current,
        errors: _,
    } = StatesDiscoverCmd::current(&mut cmd_ctx).await?;
    let state_current = states_current
        .get::<TestFileCreationShCmdState, _>(&TestFileCreationShCmdItem::ID)
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
        let mut graph_builder = ItemGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItem::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_params::<ShCmdItem<TestFileCreationShCmdItem>>(
            TestFileCreationShCmdItem::ID,
            TestFileCreationShCmdItem::params().into(),
        )
        .await?;

    let CmdOutcome {
        value: states_desired,
        errors: _,
    } = StatesDiscoverCmd::desired(&mut cmd_ctx).await?;
    let state_desired = states_desired
        .get::<State<TestFileCreationShCmdStateLogical, ShCmdExecutionRecord>, _>(
            &TestFileCreationShCmdItem::ID,
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
        let mut graph_builder = ItemGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItem::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_params::<ShCmdItem<TestFileCreationShCmdItem>>(
            TestFileCreationShCmdItem::ID,
            TestFileCreationShCmdItem::params().into(),
        )
        .await?;

    // Discover current and desired states.
    StatesDiscoverCmd::current_and_desired(&mut cmd_ctx).await?;

    // Diff current and desired states.
    let state_diffs = DiffCmd::current_and_desired(&mut cmd_ctx).await?;

    let state_diff = state_diffs
        .get::<ShCmdStateDiff, _>(&TestFileCreationShCmdItem::ID)
        .unwrap();
    assert_eq!("creation_required", state_diff.stdout());
    assert_eq!("`test_file` will be created", state_diff.stderr());

    Ok(())
}

#[tokio::test]
async fn ensure_when_creation_required_executes_apply_exec_shell_command()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItem::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_params::<ShCmdItem<TestFileCreationShCmdItem>>(
            TestFileCreationShCmdItem::ID,
            TestFileCreationShCmdItem::params().into(),
        )
        .await?;

    // Discover states current and desired
    let CmdOutcome {
        value: (states_current, _states_desired),
        errors: _,
    } = StatesDiscoverCmd::current_and_desired(&mut cmd_ctx).await?;
    let states_saved = StatesSaved::from(states_current);

    // Create the file
    let CmdOutcome {
        value: states_ensured,
        errors: _,
    } = EnsureCmd::exec(&mut cmd_ctx, &states_saved).await?;

    let state_ensured = states_ensured
        .get::<TestFileCreationShCmdState, _>(&TestFileCreationShCmdItem::ID)
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
async fn ensure_when_exists_sync_does_not_reexecute_apply_exec_shell_command()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItem::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_params::<ShCmdItem<TestFileCreationShCmdItem>>(
            TestFileCreationShCmdItem::ID,
            TestFileCreationShCmdItem::params().into(),
        )
        .await?;

    // Discover states current and desired
    let CmdOutcome {
        value: (states_current, states_desired),
        errors: _,
    } = StatesDiscoverCmd::current_and_desired(&mut cmd_ctx).await?;
    let states_saved = StatesSaved::from(states_current);

    // Create the file
    let CmdOutcome {
        value: states_ensured,
        errors: _,
    } = EnsureCmd::exec(&mut cmd_ctx, &states_saved).await?;

    // Diff state after creation
    let SingleProfileSingleFlowView {
        flow,
        params_specs,
        resources,
        ..
    } = cmd_ctx.view();
    let state_diffs = DiffCmd::diff_any(
        flow,
        params_specs,
        resources,
        &states_ensured,
        &states_desired,
    )
    .await?;

    let state_diff = state_diffs
        .get::<ShCmdStateDiff, _>(&TestFileCreationShCmdItem::ID)
        .unwrap();
    assert_eq!("exists_sync", state_diff.stdout());
    assert_eq!("nothing to do", state_diff.stderr());

    // Run again, for idempotence check
    let states_saved = StatesSavedReadCmd::exec(&mut cmd_ctx).await?;
    let CmdOutcome {
        value: states_ensured,
        errors: _,
    } = EnsureCmd::exec(&mut cmd_ctx, &states_saved).await?;

    let state_ensured = states_ensured
        .get::<TestFileCreationShCmdState, _>(&TestFileCreationShCmdItem::ID)
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
        let mut graph_builder = ItemGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItem::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_params::<ShCmdItem<TestFileCreationShCmdItem>>(
            TestFileCreationShCmdItem::ID,
            TestFileCreationShCmdItem::params().into(),
        )
        .await?;

    // Discover states current and desired
    let CmdOutcome {
        value: (states_current, _states_desired),
        errors: _,
    } = StatesDiscoverCmd::current_and_desired(&mut cmd_ctx).await?;
    let states_saved = StatesSaved::from(states_current);

    // Create the file
    EnsureCmd::exec(&mut cmd_ctx, &states_saved).await?;

    assert!(tempdir.path().join("test_file").exists());

    // Clean the file
    let states_saved = StatesSavedReadCmd::exec(&mut cmd_ctx).await?;
    CleanCmd::exec(&mut cmd_ctx, &states_saved).await?;

    assert!(!tempdir.path().join("test_file").exists());

    // Run again, for idempotence check
    let states_saved = StatesSavedReadCmd::exec(&mut cmd_ctx).await?;
    let CmdOutcome {
        value: states_cleaned,
        errors: _,
    } = CleanCmd::exec(&mut cmd_ctx, &states_saved).await?;

    let state_cleaned = states_cleaned
        .get::<TestFileCreationShCmdState, _>(&TestFileCreationShCmdItem::ID)
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
