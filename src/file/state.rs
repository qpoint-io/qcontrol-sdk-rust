//! Compatibility aliases for the generic plugin state wrapper.
//!
//! The canonical state type now lives in the crate-level `state` module so it
//! can serve file, exec, net, and HTTP callbacks without coupling the source
//! of truth to the file namespace. This module preserves the historical
//! `qcontrol::file::FileState` path for existing plugins.

pub type PluginState<'a> = crate::state::PluginState<'a>;
pub type FileState<'a> = crate::state::FileState<'a>;
