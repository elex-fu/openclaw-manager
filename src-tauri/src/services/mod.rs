//! 服务模块
//!
//! 提供核心业务逻辑服务

pub mod secure_storage;
pub mod config_manager;
pub mod process_manager;
pub mod offline_installer;
pub mod installer;
pub mod diagnostics;
pub mod log_service;
pub mod log_watcher;
pub mod plugin_manager;
pub mod plugin_market;
pub mod skill_manager;
pub mod skill_market;

#[cfg(test)]
mod process_manager_test;

#[cfg(test)]
mod plugin_manager_test;

#[cfg(test)]
mod skill_manager_test;
