use peace::{
    cfg::{
        app_name, flow_id, item_spec_id, profile, AppName, FlowId, ItemSpec, ItemSpecId, Profile,
    },
    cmd::ctx::CmdCtx,
    params::{Params, ParamsSpec, ValueResolutionCtx, ValueResolutionMode, ValueSpec},
    resources::paths::{FlowDir, ProfileDir, ProfileHistoryDir},
    rt_model::{Flow, ItemSpecGraphBuilder},
};

use crate::{
    cmd::ctx::cmd_ctx_builder::{
        assert_flow_params, assert_profile_params, assert_workspace_params, workspace,
    },
    no_op_output::NoOpOutput,
    vec_copy_item_spec::{VecA, VecAFieldWise, VecCopyItemSpec},
    PeaceTestError,
};

#[tokio::test]
async fn build() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = workspace(&tempdir, app_name!("test_single_profile_single_flow"))?;
    let profile = profile!("test_profile");
    let flow_id = flow_id!("test_flow_id");
    let flow = Flow::<PeaceTestError>::new(flow_id, ItemSpecGraphBuilder::new().build());

    let mut output = NoOpOutput;
    let cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .build()
        .await?;

    let peace_app_dir = workspace.dirs().peace_app_dir();
    let profile_dir = ProfileDir::from((peace_app_dir, &profile));
    let profile_history_dir = ProfileHistoryDir::from(&profile_dir);
    let flow_dir = FlowDir::from((&profile_dir, flow.flow_id()));

    let scope = cmd_ctx.scope();
    assert!(std::ptr::eq(&workspace, cmd_ctx.workspace()));
    assert_eq!(peace_app_dir, cmd_ctx.workspace().dirs().peace_app_dir());
    assert_eq!(&profile, scope.profile());
    assert_eq!(&profile_dir, scope.profile_dir());
    assert_eq!(&profile_history_dir, scope.profile_history_dir());
    assert_eq!(flow.flow_id(), scope.flow().flow_id());
    assert_eq!(&flow_dir, scope.flow_dir());
    Ok(())
}

#[tokio::test]
async fn build_with_workspace_params() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = workspace(&tempdir, app_name!("test_single_profile_single_flow"))?;
    let profile = profile!("test_profile");
    let flow_id = flow_id!("test_flow_id");
    let flow = Flow::<PeaceTestError>::new(flow_id, ItemSpecGraphBuilder::new().build());

    let mut output = NoOpOutput;
    let cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .with_workspace_param_value(String::from("profile"), Some(profile.clone()))
        .with_workspace_param_value(
            String::from("ws_param_1"),
            Some("ws_param_1_value".to_string()),
        )
        .build()
        .await?;

    let peace_app_dir = workspace.dirs().peace_app_dir();
    let profile_dir = ProfileDir::from((peace_app_dir, &profile));
    let profile_history_dir = ProfileHistoryDir::from(&profile_dir);
    let flow_dir = FlowDir::from((&profile_dir, flow.flow_id()));

    let scope = cmd_ctx.scope();
    let workspace_params = scope.workspace_params();
    assert!(std::ptr::eq(&workspace, cmd_ctx.workspace()));
    assert_eq!(peace_app_dir, cmd_ctx.workspace().dirs().peace_app_dir());
    assert_eq!(&profile, scope.profile());
    assert_eq!(&profile_dir, scope.profile_dir());
    assert_eq!(&profile_history_dir, scope.profile_history_dir());
    assert_eq!(flow.flow_id(), scope.flow().flow_id());
    assert_eq!(&flow_dir, scope.flow_dir());
    assert_eq!(Some(&profile), workspace_params.get("profile"));
    assert_eq!(
        Some(&"ws_param_1_value".to_string()),
        workspace_params.get("ws_param_1")
    );

    let resources = cmd_ctx.resources();
    let res_profile = &*resources.borrow::<Profile>();
    assert_eq!(&profile, res_profile);
    assert_workspace_params(resources).await?;
    Ok(())
}

#[tokio::test]
async fn build_with_profile_params() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = workspace(&tempdir, app_name!("test_single_profile_single_flow"))?;
    let profile = profile!("test_profile");
    let flow_id = flow_id!("test_flow_id");
    let flow = Flow::<PeaceTestError>::new(flow_id, ItemSpecGraphBuilder::new().build());

    let mut output = NoOpOutput;
    let cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile_param_value(String::from("profile_param_0"), Some(1u32))
        .with_profile_param_value(String::from("profile_param_1"), Some(2u64))
        .with_profile(profile.clone())
        .with_flow(&flow)
        .build()
        .await?;

    let peace_app_dir = workspace.dirs().peace_app_dir();
    let profile_dir = ProfileDir::from((peace_app_dir, &profile));
    let profile_history_dir = ProfileHistoryDir::from(&profile_dir);
    let flow_dir = FlowDir::from((&profile_dir, flow.flow_id()));

    let scope = cmd_ctx.scope();
    assert!(std::ptr::eq(&workspace, cmd_ctx.workspace()));
    assert_eq!(peace_app_dir, cmd_ctx.workspace().dirs().peace_app_dir());
    assert_eq!(&profile, scope.profile());
    assert_eq!(&profile_dir, scope.profile_dir());
    assert_eq!(&profile_history_dir, scope.profile_history_dir());
    assert_eq!(flow.flow_id(), scope.flow().flow_id());
    assert_eq!(&flow_dir, scope.flow_dir());

    let resources = cmd_ctx.resources();
    assert_profile_params(resources).await?;
    Ok(())
}

#[tokio::test]
async fn build_with_flow_params() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = workspace(&tempdir, app_name!("test_single_profile_single_flow"))?;
    let profile = profile!("test_profile");
    let flow_id = flow_id!("test_flow_id");
    let flow = Flow::<PeaceTestError>::new(flow_id, ItemSpecGraphBuilder::new().build());

    let mut output = NoOpOutput;
    let cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .with_flow_param_value(String::from("flow_param_0"), Some(true))
        .with_flow_param_value(String::from("flow_param_1"), Some(456u16))
        .build()
        .await?;

    let peace_app_dir = workspace.dirs().peace_app_dir();
    let profile_dir = ProfileDir::from((peace_app_dir, &profile));
    let profile_history_dir = ProfileHistoryDir::from(&profile_dir);
    let flow_dir = FlowDir::from((&profile_dir, flow.flow_id()));

    let scope = cmd_ctx.scope();
    let flow_params = scope.flow_params();
    assert!(std::ptr::eq(&workspace, cmd_ctx.workspace()));
    assert_eq!(peace_app_dir, cmd_ctx.workspace().dirs().peace_app_dir());
    assert_eq!(&profile, scope.profile());
    assert_eq!(&profile_dir, scope.profile_dir());
    assert_eq!(&profile_history_dir, scope.profile_history_dir());
    assert_eq!(flow.flow_id(), scope.flow().flow_id());
    assert_eq!(&flow_dir, scope.flow_dir());
    assert_eq!(Some(true), flow_params.get("flow_param_0").copied());
    assert_eq!(Some(&456u16), flow_params.get("flow_param_1"));

    let resources = cmd_ctx.resources();
    assert_flow_params(resources).await?;
    Ok(())
}

#[tokio::test]
async fn build_with_workspace_params_with_profile_params() -> Result<(), Box<dyn std::error::Error>>
{
    let tempdir = tempfile::tempdir()?;
    let workspace = workspace(&tempdir, app_name!("test_single_profile_single_flow"))?;
    let profile = profile!("test_profile");
    let flow_id = flow_id!("test_flow_id");
    let flow = Flow::<PeaceTestError>::new(flow_id, ItemSpecGraphBuilder::new().build());

    let mut output = NoOpOutput;
    let cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .with_profile_param_value(String::from("profile_param_0"), Some(1u32))
        .with_workspace_param_value(String::from("profile"), Some(profile.clone()))
        .with_profile_param_value(String::from("profile_param_1"), Some(2u64))
        .with_workspace_param_value(
            String::from("ws_param_1"),
            Some("ws_param_1_value".to_string()),
        )
        .build()
        .await?;

    let peace_app_dir = workspace.dirs().peace_app_dir();
    let profile_dir = ProfileDir::from((peace_app_dir, &profile));
    let profile_history_dir = ProfileHistoryDir::from(&profile_dir);
    let flow_dir = FlowDir::from((&profile_dir, flow.flow_id()));

    let scope = cmd_ctx.scope();
    let workspace_params = scope.workspace_params();
    let profile_params = scope.profile_params();
    assert!(std::ptr::eq(&workspace, cmd_ctx.workspace()));
    assert_eq!(peace_app_dir, cmd_ctx.workspace().dirs().peace_app_dir());
    assert_eq!(&profile, scope.profile());
    assert_eq!(&profile_dir, scope.profile_dir());
    assert_eq!(&profile_history_dir, scope.profile_history_dir());
    assert_eq!(flow.flow_id(), scope.flow().flow_id());
    assert_eq!(&flow_dir, scope.flow_dir());
    assert_eq!(Some(&profile), workspace_params.get("profile"));
    assert_eq!(
        Some(&"ws_param_1_value".to_string()),
        workspace_params.get("ws_param_1")
    );
    assert_eq!(Some(&1u32), profile_params.get("profile_param_0"));
    assert_eq!(Some(&2u64), profile_params.get("profile_param_1"));

    let resources = cmd_ctx.resources();
    let res_profile = &*resources.borrow::<Profile>();
    assert_eq!(&profile, res_profile);
    assert_workspace_params(resources).await?;
    assert_profile_params(resources).await?;
    Ok(())
}

#[tokio::test]
async fn build_with_workspace_params_with_profile_params_with_flow_params()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = workspace(&tempdir, app_name!("test_single_profile_single_flow"))?;
    let profile = profile!("test_profile");
    let flow_id = flow_id!("test_flow_id");
    let flow = Flow::<PeaceTestError>::new(flow_id, ItemSpecGraphBuilder::new().build());

    let mut output = NoOpOutput;
    let cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .with_profile_param_value(String::from("profile_param_0"), Some(1u32))
        .with_flow_param_value(String::from("flow_param_0"), Some(true))
        .with_workspace_param_value(String::from("profile"), Some(profile.clone()))
        .with_flow_param_value(String::from("flow_param_1"), Some(456u16))
        .with_profile_param_value(String::from("profile_param_1"), Some(2u64))
        .with_workspace_param_value(
            String::from("ws_param_1"),
            Some("ws_param_1_value".to_string()),
        )
        .build()
        .await?;

    let peace_app_dir = workspace.dirs().peace_app_dir();
    let profile_dir = ProfileDir::from((peace_app_dir, &profile));
    let profile_history_dir = ProfileHistoryDir::from(&profile_dir);
    let flow_dir = FlowDir::from((&profile_dir, flow.flow_id()));

    let scope = cmd_ctx.scope();
    let workspace_params = scope.workspace_params();
    let profile_params = scope.profile_params();
    let flow_params = scope.flow_params();
    assert!(std::ptr::eq(&workspace, cmd_ctx.workspace()));
    assert_eq!(peace_app_dir, cmd_ctx.workspace().dirs().peace_app_dir());
    assert_eq!(&profile, scope.profile());
    assert_eq!(&profile_dir, scope.profile_dir());
    assert_eq!(&profile_history_dir, scope.profile_history_dir());
    assert_eq!(flow.flow_id(), scope.flow().flow_id());
    assert_eq!(&flow_dir, scope.flow_dir());
    assert_eq!(Some(&profile), workspace_params.get("profile"));
    assert_eq!(
        Some(&"ws_param_1_value".to_string()),
        workspace_params.get("ws_param_1")
    );
    assert_eq!(Some(&1u32), profile_params.get("profile_param_0"));
    assert_eq!(Some(&2u64), profile_params.get("profile_param_1"));
    assert_eq!(Some(true), flow_params.get("flow_param_0").copied());
    assert_eq!(Some(&456u16), flow_params.get("flow_param_1"));

    let resources = cmd_ctx.resources();
    let res_profile = &*resources.borrow::<Profile>();
    assert_eq!(&profile, res_profile);
    assert_workspace_params(resources).await?;
    assert_profile_params(resources).await?;
    assert_flow_params(resources).await?;
    Ok(())
}

#[tokio::test]
async fn build_with_workspace_params_with_profile_from_params()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = workspace(&tempdir, app_name!("test_single_profile_single_flow"))?;
    let profile = profile!("test_profile");
    let flow_id = flow_id!("test_flow_id");
    let flow = Flow::<PeaceTestError>::new(flow_id, ItemSpecGraphBuilder::new().build());

    let mut output = NoOpOutput;
    let cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_workspace_param_value(String::from("profile"), Some(profile.clone()))
        .with_workspace_param_value(
            String::from("ws_param_1"),
            Some("ws_param_1_value".to_string()),
        )
        .with_profile_from_workspace_param(&String::from("profile"))
        .with_flow(&flow)
        .build()
        .await?;

    let peace_app_dir = workspace.dirs().peace_app_dir();
    let profile_dir = ProfileDir::from((peace_app_dir, &profile));
    let profile_history_dir = ProfileHistoryDir::from(&profile_dir);
    let flow_dir = FlowDir::from((&profile_dir, flow.flow_id()));

    let scope = cmd_ctx.scope();
    let workspace_params = scope.workspace_params();
    assert!(std::ptr::eq(&workspace, cmd_ctx.workspace()));
    assert_eq!(peace_app_dir, cmd_ctx.workspace().dirs().peace_app_dir());
    assert_eq!(&profile, scope.profile());
    assert_eq!(&profile_dir, scope.profile_dir());
    assert_eq!(&profile_history_dir, scope.profile_history_dir());
    assert_eq!(flow.flow_id(), scope.flow().flow_id());
    assert_eq!(&flow_dir, scope.flow_dir());
    assert_eq!(Some(&profile), workspace_params.get("profile"));
    assert_eq!(
        Some(&"ws_param_1_value".to_string()),
        workspace_params.get("ws_param_1")
    );

    let resources = cmd_ctx.resources();
    let res_profile = &*resources.borrow::<Profile>();
    assert_eq!(&profile, res_profile);
    assert_workspace_params(resources).await?;
    Ok(())
}

#[tokio::test]
async fn build_with_workspace_params_with_profile_params_with_profile_from_params()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = workspace(&tempdir, app_name!("test_single_profile_single_flow"))?;
    let profile = profile!("test_profile");
    let flow_id = flow_id!("test_flow_id");
    let flow = Flow::<PeaceTestError>::new(flow_id, ItemSpecGraphBuilder::new().build());

    let mut output = NoOpOutput;
    let cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile_param_value(String::from("profile_param_0"), Some(1u32))
        .with_workspace_param_value(String::from("profile"), Some(profile.clone()))
        .with_profile_param_value(String::from("profile_param_1"), Some(2u64))
        .with_workspace_param_value(
            String::from("ws_param_1"),
            Some("ws_param_1_value".to_string()),
        )
        .with_flow_param_value(String::from("flow_param_0"), Some(true))
        .with_profile_from_workspace_param(&String::from("profile"))
        .with_flow_param_value(String::from("flow_param_1"), Some(456u16))
        .with_flow(&flow)
        .build()
        .await?;

    let peace_app_dir = workspace.dirs().peace_app_dir();
    let profile_dir = ProfileDir::from((peace_app_dir, &profile));
    let profile_history_dir = ProfileHistoryDir::from(&profile_dir);
    let flow_dir = FlowDir::from((&profile_dir, flow.flow_id()));

    let scope = cmd_ctx.scope();
    let workspace_params = scope.workspace_params();
    let profile_params = scope.profile_params();
    let flow_params = scope.flow_params();
    assert!(std::ptr::eq(&workspace, cmd_ctx.workspace()));
    assert_eq!(peace_app_dir, cmd_ctx.workspace().dirs().peace_app_dir());
    assert_eq!(&profile, scope.profile());
    assert_eq!(&profile_dir, scope.profile_dir());
    assert_eq!(&profile_history_dir, scope.profile_history_dir());
    assert_eq!(flow.flow_id(), scope.flow().flow_id());
    assert_eq!(&flow_dir, scope.flow_dir());
    assert_eq!(Some(&profile), workspace_params.get("profile"));
    assert_eq!(
        Some(&"ws_param_1_value".to_string()),
        workspace_params.get("ws_param_1")
    );
    assert_eq!(Some(&1u32), profile_params.get("profile_param_0"));
    assert_eq!(Some(&2u64), profile_params.get("profile_param_1"));
    assert_eq!(Some(true), flow_params.get("flow_param_0").copied());
    assert_eq!(Some(&456u16), flow_params.get("flow_param_1"));

    let resources = cmd_ctx.resources();
    let res_profile = &*resources.borrow::<Profile>();
    assert_eq!(&profile, res_profile);
    assert_workspace_params(resources).await?;
    assert_profile_params(resources).await?;
    assert_flow_params(resources).await?;
    Ok(())
}

#[tokio::test]
async fn build_with_item_spec_params_returns_ok_when_params_provided()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = workspace(&tempdir, app_name!("test_single_profile_single_flow"))?;
    let profile = profile!("test_profile");
    let flow_id = flow_id!("test_flow_id");
    let item_spec_graph = {
        let mut item_spec_graph_builder = ItemSpecGraphBuilder::new();
        item_spec_graph_builder.add_fn(VecCopyItemSpec::default().into());
        item_spec_graph_builder.build()
    };
    let flow = Flow::<PeaceTestError>::new(flow_id, item_spec_graph);

    let mut output = NoOpOutput;
    let cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .with_item_spec_params::<VecCopyItemSpec>(
            VecCopyItemSpec::ID_DEFAULT.clone(),
            VecA(vec![1u8]).into(),
        )
        .build()
        .await?;

    let scope = cmd_ctx.scope();
    let params_specs = scope.params_specs();
    let resources = scope.resources();
    let vec_a_spec = params_specs.get::<ParamsSpec<<VecCopyItemSpec as ItemSpec>::Params<'_>>, _>(
        VecCopyItemSpec::ID_DEFAULT,
    );
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        VecCopyItemSpec::ID_DEFAULT.clone(),
        tynm::type_name::<VecA>(),
    );
    assert!(matches!(vec_a_spec,
        Some(ParamsSpec::Value { value: VecA(value) })
        if value == &[1u8]
    ));
    assert_eq!(
        Some(VecA(vec![1u8])),
        vec_a_spec.and_then(|vec_a_spec| vec_a_spec
            .resolve(resources, &mut value_resolution_ctx)
            .ok()),
    );

    Ok(())
}

#[tokio::test]
async fn build_with_item_spec_params_returns_err_when_params_not_provided_and_not_stored()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = workspace(&tempdir, app_name!("test_single_profile_single_flow"))?;
    let profile = profile!("test_profile");
    let flow_id = flow_id!("test_flow_id");
    let item_spec_graph = {
        let mut item_spec_graph_builder = ItemSpecGraphBuilder::new();
        item_spec_graph_builder.add_fn(VecCopyItemSpec::default().into());
        item_spec_graph_builder.build()
    };
    let flow = Flow::<PeaceTestError>::new(flow_id, item_spec_graph);

    let mut output = NoOpOutput;
    let cmd_ctx_result = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .build()
        .await;

    assert!(
        matches!(
            &cmd_ctx_result,
            Err(PeaceTestError::PeaceRtError(
                peace::rt_model::Error::ParamsSpecsMismatch {
                    item_spec_ids_with_no_params_specs,
                    params_specs_provided_mismatches,
                    params_specs_stored_mismatches,
                    spec_not_provided_for_previously_stored_mapping_fn,
                }
            ))
            if item_spec_ids_with_no_params_specs == &vec![VecCopyItemSpec::ID_DEFAULT.clone()]
            && params_specs_provided_mismatches.is_empty()
            && params_specs_stored_mismatches.is_none()
            && spec_not_provided_for_previously_stored_mapping_fn.is_empty(),
        ),
        "was {cmd_ctx_result:#?}"
    );

    Ok(())
}

#[tokio::test]
async fn build_with_item_spec_params_returns_ok_when_params_not_provided_but_are_stored()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = workspace(&tempdir, app_name!("test_single_profile_single_flow"))?;
    let profile = profile!("test_profile");
    let flow_id = flow_id!("test_flow_id");
    let item_spec_graph = {
        let mut item_spec_graph_builder = ItemSpecGraphBuilder::new();
        item_spec_graph_builder.add_fn(VecCopyItemSpec::default().into());
        item_spec_graph_builder.build()
    };
    let flow = Flow::<PeaceTestError>::new(flow_id, item_spec_graph);

    let mut output = NoOpOutput;
    let _cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .with_item_spec_params::<VecCopyItemSpec>(
            VecCopyItemSpec::ID_DEFAULT.clone(),
            VecA(vec![1u8]).into(),
        )
        .build()
        .await?;

    let cmd_ctx_from_stored = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .build()
        .await?;

    let scope = cmd_ctx_from_stored.scope();
    let params_specs = scope.params_specs();
    let resources = scope.resources();
    let vec_a_spec = params_specs.get::<ParamsSpec<<VecCopyItemSpec as ItemSpec>::Params<'_>>, _>(
        VecCopyItemSpec::ID_DEFAULT,
    );
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        VecCopyItemSpec::ID_DEFAULT.clone(),
        tynm::type_name::<VecA>(),
    );
    assert!(matches!(vec_a_spec,
        Some(ParamsSpec::Value { value: VecA(value) })
        if value == &[1u8]
    ));
    assert_eq!(
        Some(VecA(vec![1u8])),
        vec_a_spec.and_then(|vec_a_spec| vec_a_spec
            .resolve(resources, &mut value_resolution_ctx)
            .ok()),
    );

    Ok(())
}

#[tokio::test]
async fn build_with_item_spec_params_returns_ok_and_uses_params_provided_when_params_provided_and_stored()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = workspace(&tempdir, app_name!("test_single_profile_single_flow"))?;
    let profile = profile!("test_profile");
    let flow_id = flow_id!("test_flow_id");
    let item_spec_graph = {
        let mut item_spec_graph_builder = ItemSpecGraphBuilder::new();
        item_spec_graph_builder.add_fn(VecCopyItemSpec::default().into());
        item_spec_graph_builder.build()
    };
    let flow = Flow::<PeaceTestError>::new(flow_id, item_spec_graph);

    let mut output = NoOpOutput;
    let _cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .with_item_spec_params::<VecCopyItemSpec>(
            VecCopyItemSpec::ID_DEFAULT.clone(),
            VecA(vec![1u8]).into(),
        )
        .build()
        .await?;

    let cmd_ctx_from_stored = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .with_item_spec_params::<VecCopyItemSpec>(
            VecCopyItemSpec::ID_DEFAULT.clone(),
            VecA(vec![2u8]).into(),
        )
        .build()
        .await?;

    let scope = cmd_ctx_from_stored.scope();
    let params_specs = scope.params_specs();
    let resources = scope.resources();
    let vec_a_spec = params_specs.get::<ParamsSpec<<VecCopyItemSpec as ItemSpec>::Params<'_>>, _>(
        VecCopyItemSpec::ID_DEFAULT,
    );
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        VecCopyItemSpec::ID_DEFAULT.clone(),
        tynm::type_name::<VecA>(),
    );
    assert!(matches!(vec_a_spec,
        Some(ParamsSpec::Value { value: VecA(value) })
        if value == &[2u8]
    ));
    assert_eq!(
        Some(VecA(vec![2u8])),
        vec_a_spec.and_then(|vec_a_spec| vec_a_spec
            .resolve(resources, &mut value_resolution_ctx)
            .ok()),
    );

    Ok(())
}

#[tokio::test]
async fn build_with_item_spec_params_returns_err_when_params_provided_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = workspace(&tempdir, app_name!("test_single_profile_single_flow"))?;
    let profile = profile!("test_profile");
    let flow_id = flow_id!("test_flow_id");
    let item_spec_graph = {
        let mut item_spec_graph_builder = ItemSpecGraphBuilder::new();
        item_spec_graph_builder.add_fn(VecCopyItemSpec::default().into());
        item_spec_graph_builder.build()
    };
    let flow = Flow::<PeaceTestError>::new(flow_id, item_spec_graph);

    let mut output = NoOpOutput;
    let _cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .with_item_spec_params::<VecCopyItemSpec>(
            VecCopyItemSpec::ID_DEFAULT.clone(),
            VecA(vec![1u8]).into(),
        )
        .build()
        .await?;

    let cmd_ctx_result = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .with_item_spec_params::<VecCopyItemSpec>(
            item_spec_id!("mismatch_id"),
            VecA(vec![2u8]).into(),
        )
        .build()
        .await;

    assert!(
        matches!(
            &cmd_ctx_result,
            Err(PeaceTestError::PeaceRtError(
                peace::rt_model::Error::ParamsSpecsMismatch {
                    item_spec_ids_with_no_params_specs,
                    params_specs_provided_mismatches,
                    params_specs_stored_mismatches,
                    spec_not_provided_for_previously_stored_mapping_fn,
                }
            ))
            if item_spec_ids_with_no_params_specs.is_empty()
            && matches!(
                params_specs_provided_mismatches.get(&item_spec_id!("mismatch_id")),
                Some(ParamsSpec::Value { value: VecA(value) })
                if value == &vec![2u8]
            )
            && matches!(
                params_specs_stored_mismatches,
                Some(params_specs_stored_mismatches)
                if params_specs_stored_mismatches.is_empty()
            )
            && spec_not_provided_for_previously_stored_mapping_fn.is_empty(),
        ),
        "was {cmd_ctx_result:#?}"
    );

    Ok(())
}

#[tokio::test]
async fn build_with_item_spec_params_returns_err_when_params_stored_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = workspace(&tempdir, app_name!("test_single_profile_single_flow"))?;
    let profile = profile!("test_profile");
    let flow_id = flow_id!("test_flow_id");
    let item_spec_graph = {
        let mut item_spec_graph_builder = ItemSpecGraphBuilder::new();
        item_spec_graph_builder.add_fn(VecCopyItemSpec::new(item_spec_id!("original_id")).into());
        item_spec_graph_builder.build()
    };
    let flow = Flow::<PeaceTestError>::new(flow_id.clone(), item_spec_graph);

    let mut output = NoOpOutput;
    let _cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .with_item_spec_params::<VecCopyItemSpec>(
            item_spec_id!("original_id"),
            VecA(vec![1u8]).into(),
        )
        .build()
        .await?;

    let item_spec_graph = {
        let mut item_spec_graph_builder = ItemSpecGraphBuilder::new();
        // Without the original ID also registered for the item spec,
        // item_spec_params deserialization will fail before reaching the params merge
        // error.
        item_spec_graph_builder.add_fn(VecCopyItemSpec::new(item_spec_id!("original_id")).into());
        item_spec_graph_builder.add_fn(VecCopyItemSpec::new(item_spec_id!("new_id")).into());
        item_spec_graph_builder.build()
    };
    let flow = Flow::<PeaceTestError>::new(flow_id, item_spec_graph);
    let cmd_ctx_result = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .with_item_spec_params::<VecCopyItemSpec>(
            item_spec_id!("mismatch_id"),
            VecA(vec![2u8]).into(),
        )
        .build()
        .await;

    assert!(
        matches!(
            &cmd_ctx_result,
            Err(PeaceTestError::PeaceRtError(
                peace::rt_model::Error::ParamsSpecsMismatch {
                    item_spec_ids_with_no_params_specs,
                    params_specs_provided_mismatches,
                    params_specs_stored_mismatches,
                    spec_not_provided_for_previously_stored_mapping_fn,
                }
            ))
            if item_spec_ids_with_no_params_specs == &vec![item_spec_id!("new_id")]
            && matches!(
                params_specs_provided_mismatches.get(&item_spec_id!("mismatch_id")),
                Some(ParamsSpec::Value { value: VecA(value) })
                if value == &vec![2u8]
            )
            && matches!(
                params_specs_stored_mismatches,
                Some(params_specs_stored_mismatches)
                if params_specs_stored_mismatches.is_empty()
            )
            && spec_not_provided_for_previously_stored_mapping_fn.is_empty(),
        ),
        "was {cmd_ctx_result:#?}"
    );

    Ok(())
}

#[tokio::test]
async fn build_with_item_spec_params_returns_ok_when_spec_provided_for_previous_mapping_fn()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = workspace(&tempdir, app_name!("test_single_profile_single_flow"))?;
    let profile = profile!("test_profile");
    let flow_id = flow_id!("test_flow_id");
    let item_spec_graph = {
        let mut item_spec_graph_builder = ItemSpecGraphBuilder::new();
        item_spec_graph_builder
            .add_fn(VecCopyItemSpec::new(VecCopyItemSpec::ID_DEFAULT.clone()).into());
        item_spec_graph_builder.build()
    };
    let flow = Flow::<PeaceTestError>::new(flow_id.clone(), item_spec_graph);

    let mut output = NoOpOutput;
    let _cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .with_item_spec_params::<VecCopyItemSpec>(
            VecCopyItemSpec::ID_DEFAULT.clone(),
            VecA::field_wise_spec()
                .with_0_from_map(|_: &u8| Some(vec![1u8]))
                .build(),
        )
        .build()
        .await?;

    let item_spec_graph = {
        let mut item_spec_graph_builder = ItemSpecGraphBuilder::new();
        item_spec_graph_builder
            .add_fn(VecCopyItemSpec::new(VecCopyItemSpec::ID_DEFAULT.clone()).into());
        item_spec_graph_builder.build()
    };
    let flow = Flow::<PeaceTestError>::new(flow_id, item_spec_graph);
    let cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .with_item_spec_params::<VecCopyItemSpec>(
            VecCopyItemSpec::ID_DEFAULT.clone(),
            VecA::field_wise_spec()
                .with_0_from_map(|_: &u8| Some(vec![1u8]))
                .build(),
        )
        .with_flow_param_value(String::from("for_item_spec_mapping"), Some(1u8))
        .build()
        .await?;

    let scope = cmd_ctx.scope();
    let params_specs = scope.params_specs();
    let resources = scope.resources();
    let vec_a_spec = params_specs.get::<ParamsSpec<<VecCopyItemSpec as ItemSpec>::Params<'_>>, _>(
        VecCopyItemSpec::ID_DEFAULT,
    );
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        VecCopyItemSpec::ID_DEFAULT.clone(),
        tynm::type_name::<VecA>(),
    );
    assert!(
        matches!(vec_a_spec,
            Some(ParamsSpec::FieldWise {
                field_wise_spec: VecAFieldWise(ValueSpec::<Vec<u8>>::MappingFn(mapping_fn)),
            })
            if mapping_fn.is_valued()
        ),
        "was {vec_a_spec:?}"
    );
    assert_eq!(
        Some(VecA(vec![1u8])),
        vec_a_spec.and_then(|vec_a_spec| vec_a_spec
            .resolve(resources, &mut value_resolution_ctx)
            .ok()),
    );

    Ok(())
}

#[tokio::test]
async fn build_with_item_spec_params_returns_err_when_spec_not_provided_for_previous_mapping_fn()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = workspace(&tempdir, app_name!("test_single_profile_single_flow"))?;
    let profile = profile!("test_profile");
    let flow_id = flow_id!("test_flow_id");
    let item_spec_graph = {
        let mut item_spec_graph_builder = ItemSpecGraphBuilder::new();
        item_spec_graph_builder
            .add_fn(VecCopyItemSpec::new(VecCopyItemSpec::ID_DEFAULT.clone()).into());
        item_spec_graph_builder.build()
    };
    let flow = Flow::<PeaceTestError>::new(flow_id.clone(), item_spec_graph);

    let mut output = NoOpOutput;
    let _cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .with_item_spec_params::<VecCopyItemSpec>(
            VecCopyItemSpec::ID_DEFAULT.clone(),
            VecA::field_wise_spec()
                .with_0_from_map(|_: &u8| Some(vec![1u8]))
                .build(),
        )
        .build()
        .await?;

    let item_spec_graph = {
        let mut item_spec_graph_builder = ItemSpecGraphBuilder::new();
        item_spec_graph_builder
            .add_fn(VecCopyItemSpec::new(VecCopyItemSpec::ID_DEFAULT.clone()).into());
        item_spec_graph_builder.build()
    };
    let flow = Flow::<PeaceTestError>::new(flow_id, item_spec_graph);
    let cmd_ctx_result = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        // Note: no item_spec_params for `original_id`
        .build()
        .await;

    assert!(
        matches!(
            &cmd_ctx_result,
            Err(PeaceTestError::PeaceRtError(
                peace::rt_model::Error::ParamsSpecsMismatch {
                    item_spec_ids_with_no_params_specs,
                    params_specs_provided_mismatches,
                    params_specs_stored_mismatches,
                    spec_not_provided_for_previously_stored_mapping_fn,
                }
            ))
            if item_spec_ids_with_no_params_specs.is_empty()
            && params_specs_provided_mismatches.is_empty()
            && matches!(
                params_specs_stored_mismatches,
                Some(params_specs_stored_mismatches)
                if params_specs_stored_mismatches.is_empty()
            )
            && spec_not_provided_for_previously_stored_mapping_fn == &[VecCopyItemSpec::ID_DEFAULT.clone()],
        ),
        "was {cmd_ctx_result:#?}"
    );

    Ok(())
}

#[tokio::test]
async fn build_with_item_spec_params_returns_deserialization_err_when_item_spec_renamed()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = workspace(&tempdir, app_name!("test_single_profile_single_flow"))?;
    let profile = profile!("test_profile");
    let flow_id = flow_id!("test_flow_id");
    let item_spec_graph = {
        let mut item_spec_graph_builder = ItemSpecGraphBuilder::new();
        item_spec_graph_builder.add_fn(VecCopyItemSpec::new(item_spec_id!("original_id")).into());
        item_spec_graph_builder.build()
    };
    let flow = Flow::<PeaceTestError>::new(flow_id.clone(), item_spec_graph);

    let mut output = NoOpOutput;
    let _cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .with_item_spec_params::<VecCopyItemSpec>(
            item_spec_id!("original_id"),
            VecA(vec![1u8]).into(),
        )
        .build()
        .await?;

    let item_spec_graph = {
        let mut item_spec_graph_builder = ItemSpecGraphBuilder::new();
        // Note: no `"original_id"` item spec.
        item_spec_graph_builder.add_fn(VecCopyItemSpec::new(item_spec_id!("new_id")).into());
        item_spec_graph_builder.build()
    };
    let flow = Flow::<PeaceTestError>::new(flow_id, item_spec_graph);
    let cmd_ctx_result = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile.clone())
        .with_flow(&flow)
        .with_item_spec_params::<VecCopyItemSpec>(
            item_spec_id!("mismatch_id"),
            VecA(vec![2u8]).into(),
        )
        .build()
        .await;

    assert!(
        matches!(
            &cmd_ctx_result,
            Err(PeaceTestError::PeaceRtError(
                peace::rt_model::Error::ParamsSpecsDeserialize {
                    profile,
                    flow_id,
                    #[cfg(feature = "error_reporting")]
                    params_specs_file_source: _,
                    #[cfg(feature = "error_reporting")]
                    error_span: _,
                    #[cfg(feature = "error_reporting")]
                    error_message: _,
                    #[cfg(feature = "error_reporting")]
                    context_span: _,
                    error: _,
                }
            ))
            if profile == &profile!("test_profile")
            && flow_id == &flow_id!("test_flow_id")
        ),
        "was {cmd_ctx_result:#?}"
    );

    Ok(())
}
