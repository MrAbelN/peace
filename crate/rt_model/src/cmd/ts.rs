//! Type states for a [`CmdContext`].
//!
//! These distinguish how a `CmdContext` is prepared for use.
//!
//! [`CmdContext`]: crate::workspace::CmdContext

use serde::{Deserialize, Serialize};

#[rustfmt::skip]
// # Idea
//
// /// Marks a workspace with no `.peace` directory, so no parameters or metadata
// /// are persisted.
// ///
// /// All state information (current, desired, diff) must be discovered within a
// /// command execution.
// ///
// /// * Does not write parameters or metadata to disk -- command executions cannot
// ///   reason off previous executions.
// /// * Limits usability in terms of parameters, and caching current and desired
// ///   states.
// #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
// pub struct WithoutPeaceDir;

/// CmdContext with no profile selected.
///
/// This is used when a command is run using a shared workspace profile and flow.
///
/// # Examples
///
/// * Downloading a repository or project, used to deploy different profile environments.
/// * Storing preferences for a user.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct CmdContextCommon;

/// CmdContext with a profile selected.
///
/// This is used when a command is run for a particular profile, and the
/// information is either not applicable to any flow, or is shared across flows.
///
/// # Examples
///
/// * Storing information for a profile.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ProfileSelected;

/// CmdContext with profile selection deferred.
///
/// This is used when the profile is evaluated at the point of command context
/// building, for example when the profile is read from workflow params.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ProfileDeferred;

/// CmdContext with a flow selected.
///
/// This is used when a command is run for a particular flow.
///
/// # Examples
///
/// * An environment deployment workflow.
/// * A configuration management workflow.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct FlowIdSelected;