//! Runtime data types for the peace automation framework.
//!
//! This crate re-exports types from `peace_rt_model_native` or
//! `peace_rt_model_web` depending on the compilation target architecture.

// Re-exports
pub use fn_graph::{self, FnRef, FnRefMut};
pub use peace_rt_model_core::{
    cmd_context_params, OutputFormat, OutputFormatParseError, OutputWrite,
};

#[cfg(not(target_arch = "wasm32"))]
pub use peace_rt_model_native::{
    CliOutput, Error, NativeStorage as Storage, SyncIoBridge, Workspace, WorkspaceDirsBuilder,
    WorkspaceInitializer, WorkspaceSpec,
};

#[cfg(all(not(target_arch = "wasm32"), feature = "output_progress"))]
pub use peace_rt_model_native::{CliProgressFormat, CliProgressFormatParseError};

#[cfg(target_arch = "wasm32")]
pub use peace_rt_model_web::{
    Error, WebStorage as Storage, Workspace, WorkspaceDirsBuilder, WorkspaceInitializer,
    WorkspaceSpec,
};

pub use crate::{
    cmd_context::CmdContext, cmd_context_builder::CmdContextBuilder,
    in_memory_text_output::InMemoryTextOutput, item_spec_boxed::ItemSpecBoxed,
    item_spec_graph::ItemSpecGraph, item_spec_graph_builder::ItemSpecGraphBuilder,
    item_spec_rt::ItemSpecRt, item_spec_wrapper::ItemSpecWrapper,
    states_serializer::StatesSerializer, states_type_regs::StatesTypeRegs,
};

pub mod outcomes;

mod cmd_context;
mod cmd_context_builder;
mod in_memory_text_output;
mod item_spec_boxed;
mod item_spec_graph;
mod item_spec_graph_builder;
mod item_spec_rt;
mod item_spec_wrapper;
mod states_serializer;
mod states_type_regs;

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        pub use crate::cmd_progress_tracker::CmdProgressTracker;

        mod cmd_progress_tracker;
    }
}
