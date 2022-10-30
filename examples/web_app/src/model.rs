//! Data structures

pub use self::{
    env_type::EnvType, env_type_parse_error::EnvTypeParseError, repo_slug::RepoSlug,
    repo_slug_error::RepoSlugError, web_app_error::WebAppError,
};

#[cfg(not(target_arch = "wasm32"))]
pub mod cli_args;

mod env_type;
mod env_type_parse_error;
mod repo_slug;
mod repo_slug_error;
mod web_app_error;
