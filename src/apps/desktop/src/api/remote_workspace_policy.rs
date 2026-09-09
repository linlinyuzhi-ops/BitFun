//! Desktop closure test for the Product Operation Registry.
//!
//! Every Tauri command registered in `lib.rs` (`tauri::generate_handler!`)
//! must have exactly one row in
//! `openbitfun_product_domains::remote_surface`, which declares how the command
//! behaves for remote SSH/Docker workspaces and in Peer Device Mode. Remote SSH
//! workspaces have no central command router: each handler adapts itself
//! (usually through `resolve_desktop_path_target` / `lookup_remote_connection`
//! / `is_remote_path`), so the registry is what forces a new command to declare
//! its remote behavior at all. Historically the lack of that declaration
//! produced silent local/remote feature gaps (for example the PR reviewer
//! opening to a blank panel in remote workspaces).
//!
//! The registry owns the stances, the frozen `Unaudited` backlog, and the
//! ratchet tests; this module only proves the desktop registration set and the
//! registry's `TauriCommand` rows are the same set. See
//! `docs/architecture/remote-surface-contract.md`.

/// How a Tauri command behaves for remote SSH workspaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteWorkspacePolicy {
    /// Routed to the remote host for remote workspace paths/sessions.
    RemoteRouted,
    /// Explicitly rejected for remote workspaces with a clear error.
    RemoteUnsupported,
    /// Intentionally local-host behavior regardless of workspace.
    LocalOnly,
    /// Independent of workspace filesystem location.
    WorkspaceAgnostic,
    /// Frozen backlog; must not grow. See module docs.
    LegacyUnaudited,
}

/// Declared remote-workspace policy for every registered Tauri command.
pub const REMOTE_WORKSPACE_COMMAND_POLICIES: &[(&str, RemoteWorkspacePolicy)] = &[
    ("accept_file", RemoteWorkspacePolicy::LegacyUnaudited),
    ("accept_operation", RemoteWorkspacePolicy::LegacyUnaudited),
    ("accept_session", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "account_auto_sync",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "account_connect_devices",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "account_delegate_to_paired",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "account_delete_device",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "account_delete_synced_session",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "account_device_rpc",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "account_execute_on_device",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "account_export_all_sessions",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "account_export_local_session",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "account_fetch_session_turns",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "account_fetch_settings",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "account_fetch_synced_sessions",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "account_get_credential_hint",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "account_import_remote_sessions",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "account_list_devices",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("account_login", RemoteWorkspacePolicy::WorkspaceAgnostic),
    (
        "account_finalize_login",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "account_cancel_pending_login",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("account_logout", RemoteWorkspacePolicy::WorkspaceAgnostic),
    (
        "account_online_devices",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "account_send_session_to_device",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("account_status", RemoteWorkspacePolicy::WorkspaceAgnostic),
    (
        "account_sync_session",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "account_sync_settings",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "append_flow_chat_diagnostics",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "appearance_market_browse",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "appearance_market_download_release",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "appearance_market_get_listing",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "appearance_market_get_review_submission",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "appearance_market_list_review_submissions",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "appearance_market_list_submissions",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "appearance_market_submit_package",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "appearance_market_review_submission",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "appearance_market_withdraw_submission",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "account_token_expired",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "activate_session_goal",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("add_skill", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "apply_external_mcp_import_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "apply_external_source_control_action_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "get_external_ecosystem_awareness_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "acknowledge_external_ecosystems_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    ("apply_patch", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "archive_all_sessions",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("archive_session", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "browser_control_enable_default_cdp",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "browser_control_get_status",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    ("browser_control_launch", RemoteWorkspacePolicy::LocalOnly),
    (
        "browser_control_list_browsers",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "browser_control_restart_with_cdp",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    ("browser_get_url", RemoteWorkspacePolicy::LocalOnly),
    ("browser_webview_create", RemoteWorkspacePolicy::LocalOnly),
    ("browser_webview_eval", RemoteWorkspacePolicy::LocalOnly),
    ("browser_webview_navigate", RemoteWorkspacePolicy::LocalOnly),
    ("browser_webview_reload", RemoteWorkspacePolicy::LocalOnly),
    (
        "browser_webview_set_bounds",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    ("btw_ask_stream", RemoteWorkspacePolicy::RemoteRouted),
    ("btw_cancel", RemoteWorkspacePolicy::RemoteRouted),
    (
        "cancel_acp_dialog_turn",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("cancel_dialog_turn", RemoteWorkspacePolicy::LegacyUnaudited),
    ("interrupt_dialog_turn", RemoteWorkspacePolicy::LocalOnly),
    (
        "cancel_insights_generation",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "cancel_mcp_remote_oauth",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("cancel_search", RemoteWorkspacePolicy::LegacyUnaudited),
    ("cancel_session", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "cancel_subscription_login",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    ("cancel_tool", RemoteWorkspacePolicy::LegacyUnaudited),
    ("cancel_transfer", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "canonicalize_agent_profile_configs",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "check_command_exists",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "check_commands_exist",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "check_for_updates",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "check_git_isolation",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("check_path_exists", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "choose_external_mcp_conflict_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "choose_external_subagent_conflict_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "cleanup_invalid_workspaces",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("cleanup_storage", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "cleanup_storage_with_policy",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "clear_mcp_remote_auth",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "clear_session_thread_goal",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("close_workspace", RemoteWorkspacePolicy::LegacyUnaudited),
    ("compact_session", RemoteWorkspacePolicy::LegacyUnaudited),
    ("compress_path", RemoteWorkspacePolicy::RemoteRouted),
    ("compute_diff", RemoteWorkspacePolicy::LegacyUnaudited),
    ("computer_use_get_status", RemoteWorkspacePolicy::LocalOnly),
    (
        "computer_use_open_system_settings",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "computer_use_request_permissions",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "control_background_command",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "control_deep_review_queue",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "create_acp_flow_session",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "create_assistant_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("create_cron_job", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "create_custom_agent",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("create_directory", RemoteWorkspacePolicy::LegacyUnaudited),
    ("create_file", RemoteWorkspacePolicy::LegacyUnaudited),
    ("create_miniapp", RemoteWorkspacePolicy::LegacyUnaudited),
    ("create_session", RemoteWorkspacePolicy::LegacyUnaudited),
    ("create_subagent", RemoteWorkspacePolicy::LegacyUnaudited),
    ("debug_close_devtools", RemoteWorkspacePolicy::LocalOnly),
    ("debug_devtools_available", RemoteWorkspacePolicy::LocalOnly),
    ("debug_element_picked", RemoteWorkspacePolicy::LocalOnly),
    ("debug_open_devtools", RemoteWorkspacePolicy::LocalOnly),
    ("decompress_path", RemoteWorkspacePolicy::RemoteRouted),
    (
        "delete_agent_companion_pet_package",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "delete_all_archived_sessions",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "delete_assistant_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("delete_cron_job", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "delete_custom_agent",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("delete_directory", RemoteWorkspacePolicy::LegacyUnaudited),
    ("delete_file", RemoteWorkspacePolicy::LegacyUnaudited),
    ("delete_mcp_server", RemoteWorkspacePolicy::LegacyUnaudited),
    ("delete_miniapp", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "delete_persisted_session",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("delete_session", RemoteWorkspacePolicy::LegacyUnaudited),
    ("delete_skill", RemoteWorkspacePolicy::LegacyUnaudited),
    ("delete_subagent", RemoteWorkspacePolicy::LegacyUnaudited),
    // Detached dispatch is routed by its own immutable target and observer
    // index, never by the currently open workspace.
    ("dispatch_cancel", RemoteWorkspacePolicy::WorkspaceAgnostic),
    (
        "dispatch_install_cli_cancel",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "dispatch_install_cli_poll",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "dispatch_install_cli_start",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "dispatch_provision_target",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "dispatch_list_jobs",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "dispatch_sync_result",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "dispatch_list_targets",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "dispatch_probe_target",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "dispatch_sync_model_config",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("dispatch_answer", RemoteWorkspacePolicy::WorkspaceAgnostic),
    ("dispatch_append", RemoteWorkspacePolicy::WorkspaceAgnostic),
    (
        "dispatch_continue",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "dispatch_load_transcript",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "dispatch_save_transcript",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("dispatch_status", RemoteWorkspacePolicy::WorkspaceAgnostic),
    ("dispatch_query", RemoteWorkspacePolicy::WorkspaceAgnostic),
    ("dispatch_submit", RemoteWorkspacePolicy::WorkspaceAgnostic),
    (
        "dismiss_announcement",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "download_skill_market",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("editor_ai_cancel", RemoteWorkspacePolicy::LegacyUnaudited),
    ("editor_ai_stream", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "ensure_assistant_bootstrap",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "ensure_coordinator_session",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("execute_tool", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "expand_external_prompt_command_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    ("explorer_get_children", RemoteWorkspacePolicy::RemoteRouted),
    (
        "explorer_get_children_paginated",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    (
        "explorer_get_file_tree",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("export_config", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "export_diagnostics_bundle",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "export_local_file_to_path",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "export_session_transcript",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "fetch_mcp_app_resource",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("fork_session", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "generate_commit_message",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("generate_insights", RemoteWorkspacePolicy::RemoteRouted),
    (
        "get_token_usage_statistics",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "generate_session_title",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("get_acp_clients", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "get_acp_session_commands",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_acp_session_options",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_agent_profile_config",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_agent_profile_configs",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_ai_model_catalog",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "project_ai_model_reasoning_catalog",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "get_models_dev_catalog_status",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "get_all_modified_files",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("get_all_tools_info", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "get_announcement_tips",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("get_app_state", RemoteWorkspacePolicy::WorkspaceAgnostic),
    ("get_app_version", RemoteWorkspacePolicy::WorkspaceAgnostic),
    (
        "get_available_modes",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_available_tools",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_baseline_snapshot_diff",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_clipboard_files",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("get_config", RemoteWorkspacePolicy::LegacyUnaudited),
    ("get_configs", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "get_current_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_custom_agent_detail",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_default_review_team_definition",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_directory_children",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    (
        "get_directory_children_paginated",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    (
        "get_external_hook_catalog",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "get_external_hook_import_snapshot",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "plan_external_hook_import_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "apply_external_hook_import_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "mutate_external_hook_import_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "get_external_source_snapshot",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "get_workspace_reference_snapshot",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "get_native_prompt_command_conflicts_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "reveal_external_source_location",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "get_external_source_control_snapshot",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "get_file_change_history",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("get_file_diff", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "get_file_editor_sync_hash",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("get_file_metadata", RemoteWorkspacePolicy::LegacyUnaudited),
    ("get_file_tree", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "get_global_config_health",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_global_config_status",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_global_skill_settings",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "get_health_status",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("get_latest_insights", RemoteWorkspacePolicy::LocalOnly),
    ("get_mcp_prompt", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "get_mcp_remote_oauth_session",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_mcp_server_status",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("get_mcp_servers", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "get_mcp_tool_ui_uri",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("get_memory_paths", RemoteWorkspacePolicy::LegacyUnaudited),
    ("get_miniapp", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "get_miniapp_draft_storage",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_miniapp_storage",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_miniapp_versions",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_mode_skill_configs",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("get_model_configs", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "get_opened_workspaces",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_primary_assistant_workspace",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("get_operation_diff", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "get_operation_summary",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_pending_announcements",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "get_project_storage_paths",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_readonly_tools_info",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_recent_workspaces",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_runtime_capabilities",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_runtime_logging_info",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_session_file_diff_stats",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("get_session_lineage", RemoteWorkspacePolicy::RemoteRouted),
    (
        "get_session_permission_mode",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    ("get_session_files", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "get_session_operations",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("get_session_stats", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "get_session_thread_goal",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("get_session_turns", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "get_session_usage_report",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("get_skill_configs", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "get_snapshot_sessions",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_snapshot_system_stats",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_startup_native_trace",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("get_statistics", RemoteWorkspacePolicy::LegacyUnaudited),
    ("get_storage_paths", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "get_storage_statistics",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_subagent_detail",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "get_subscription_login_status",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "get_prevent_sleep_enabled",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    ("get_system_info", RemoteWorkspacePolicy::WorkspaceAgnostic),
    ("get_tool_info", RemoteWorkspacePolicy::LegacyUnaudited),
    ("get_turn_files", RemoteWorkspacePolicy::LegacyUnaudited),
    ("get_watched_paths", RemoteWorkspacePolicy::LegacyUnaudited),
    ("git_add_files", RemoteWorkspacePolicy::RemoteRouted),
    ("git_add_worktree", RemoteWorkspacePolicy::RemoteUnsupported),
    ("git_checkout_branch", RemoteWorkspacePolicy::RemoteRouted),
    ("git_cherry_pick", RemoteWorkspacePolicy::RemoteRouted),
    ("git_cherry_pick_abort", RemoteWorkspacePolicy::RemoteRouted),
    (
        "git_cherry_pick_continue",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    ("git_commit", RemoteWorkspacePolicy::RemoteRouted),
    ("git_create_branch", RemoteWorkspacePolicy::RemoteRouted),
    ("git_delete_branch", RemoteWorkspacePolicy::RemoteRouted),
    ("git_get_branches", RemoteWorkspacePolicy::RemoteRouted),
    ("git_get_changed_files", RemoteWorkspacePolicy::RemoteRouted),
    ("git_get_commits", RemoteWorkspacePolicy::RemoteRouted),
    ("git_get_diff", RemoteWorkspacePolicy::RemoteRouted),
    (
        "git_get_enhanced_branches",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    ("git_get_file_content", RemoteWorkspacePolicy::RemoteRouted),
    ("git_get_graph", RemoteWorkspacePolicy::RemoteUnsupported),
    ("git_get_repository", RemoteWorkspacePolicy::RemoteRouted),
    (
        "git_get_repository_basic",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    (
        "git_get_repository_trust",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    ("git_get_status", RemoteWorkspacePolicy::RemoteRouted),
    ("git_is_repository", RemoteWorkspacePolicy::RemoteRouted),
    (
        "git_list_worktrees",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    ("git_pull", RemoteWorkspacePolicy::RemoteRouted),
    ("git_push", RemoteWorkspacePolicy::RemoteRouted),
    (
        "git_remove_worktree",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    ("git_reset_files", RemoteWorkspacePolicy::RemoteRouted),
    ("git_reset_to_commit", RemoteWorkspacePolicy::RemoteRouted),
    ("git_resolve_revision", RemoteWorkspacePolicy::RemoteRouted),
    ("git_trust_repository", RemoteWorkspacePolicy::RemoteRouted),
    ("grant_miniapp_path", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "grant_miniapp_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("has_insights_data", RemoteWorkspacePolicy::RemoteRouted),
    (
        "hide_agent_companion_desktop_pet",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "hide_main_window_after_close_request",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    ("i18n_get_config", RemoteWorkspacePolicy::WorkspaceAgnostic),
    (
        "i18n_get_current_language",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "i18n_get_supported_languages",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("i18n_set_config", RemoteWorkspacePolicy::WorkspaceAgnostic),
    (
        "i18n_set_language",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "import_agent_companion_pet_package",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("import_config", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "initialize_acp_clients",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("initialize_ai", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "initialize_mcp_servers",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "initialize_mcp_servers_non_destructive",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "initialize_project_storage",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "initialize_snapshot",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "initialize_tray_after_startup",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "initialize_workspace_startup_state",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "install_acp_client_cli",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("install_update", RemoteWorkspacePolicy::WorkspaceAgnostic),
    (
        "list_agent_companion_pets",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "list_agent_tool_names",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "list_ai_models_by_config",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "list_archived_sessions",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "list_background_command_activities",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "list_pending_permission_requests",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "list_project_permission_grants",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "list_project_permission_audit",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "get_project_permission_rules",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    (
        "save_project_permission_rules",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    ("list_cron_jobs", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "list_directory_files",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "list_manageable_subagents",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    ("list_mcp_prompts", RemoteWorkspacePolicy::LegacyUnaudited),
    ("list_mcp_resources", RemoteWorkspacePolicy::LegacyUnaudited),
    ("list_miniapps", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "list_persisted_sessions",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "list_persisted_sessions_page",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("list_sessions", RemoteWorkspacePolicy::LegacyUnaudited),
    ("list_skill_market", RemoteWorkspacePolicy::LegacyUnaudited),
    ("list_subagents", RemoteWorkspacePolicy::RemoteRouted),
    (
        "list_subscription_accounts",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "list_visible_subagents",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    (
        "load_acp_json_config",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "load_canvas_artifact",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("load_canvas_state", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "load_git_repo_history",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("load_insights_report", RemoteWorkspacePolicy::LocalOnly),
    (
        "load_mcp_json_config",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "load_persisted_session_metadata",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "load_session_event_backfill",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "load_session_turn_window",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    ("load_session_turns", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "logout_subscription_account",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "lsp_change_document",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("lsp_close_document", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "lsp_close_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("lsp_detect_project", RemoteWorkspacePolicy::LegacyUnaudited),
    ("lsp_did_change", RemoteWorkspacePolicy::LegacyUnaudited),
    ("lsp_did_close", RemoteWorkspacePolicy::LegacyUnaudited),
    ("lsp_did_open", RemoteWorkspacePolicy::LegacyUnaudited),
    ("lsp_did_save", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "lsp_find_references",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "lsp_find_references_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "lsp_format_document",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "lsp_format_document_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "lsp_get_all_server_states",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "lsp_get_code_actions_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "lsp_get_completions",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "lsp_get_completions_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "lsp_get_document_highlight_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "lsp_get_document_symbols_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("lsp_get_hover", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "lsp_get_hover_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "lsp_get_inlay_hints_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("lsp_get_plugin", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "lsp_get_semantic_tokens_range_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "lsp_get_semantic_tokens_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "lsp_get_server_capabilities",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "lsp_get_server_state",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "lsp_get_supported_extensions",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "lsp_goto_definition",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "lsp_goto_definition_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("lsp_initialize", RemoteWorkspacePolicy::LegacyUnaudited),
    ("lsp_install_plugin", RemoteWorkspacePolicy::LegacyUnaudited),
    ("lsp_list_plugins", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "lsp_list_workspaces",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("lsp_open_document", RemoteWorkspacePolicy::LegacyUnaudited),
    ("lsp_open_workspace", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "lsp_prestart_server",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "lsp_rename_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("lsp_save_document", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "lsp_start_server_for_file",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "lsp_stop_all_servers",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("lsp_stop_server", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "lsp_stop_server_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "lsp_uninstall_plugin",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "mark_announcement_seen",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "miniapp_agent_cancel",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_agent_cancel_stale_runs",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_agent_ensure_session",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    ("miniapp_agent_run", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "miniapp_agent_turn_text",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("miniapp_ai_cancel", RemoteWorkspacePolicy::LegacyUnaudited),
    ("miniapp_ai_chat", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "miniapp_ai_complete",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_ai_list_models",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_apply_draft",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_create_draft",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_decline_builtin_update",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_dialog_message",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_discard_draft",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_draft_host_call",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_draft_worker_call",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_draft_worker_stop",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_get_customization_metadata",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("miniapp_get_draft", RemoteWorkspacePolicy::LegacyUnaudited),
    ("miniapp_host_call", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "miniapp_import_from_path",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_market_auth_poll",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "miniapp_market_auth_start",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "miniapp_market_browse",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "miniapp_market_capture_window",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "miniapp_market_get_listing",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "miniapp_market_import_package",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "miniapp_market_inspect_package",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    ("miniapp_market_install", RemoteWorkspacePolicy::LocalOnly),
    (
        "miniapp_market_installed_origins",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "miniapp_market_installed_status",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "miniapp_market_list_submissions",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "miniapp_market_logout",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "miniapp_market_me",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "miniapp_market_set_favorite",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "miniapp_market_set_rating",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "miniapp_market_submit_installed",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "miniapp_market_withdraw_submission",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "miniapp_install_deps",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_permission_diff_for_draft",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("miniapp_recompile", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "miniapp_render_slide_page",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_runtime_status",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_set_draft_permissions",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_sync_draft_from_fs",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_sync_from_fs",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_worker_call",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_worker_list_running",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "miniapp_worker_stop",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("minimize_to_tray", RemoteWorkspacePolicy::LocalOnly),
    (
        "never_show_announcement",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "notify_cron_host_ready",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "open_html_file_in_browser",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    ("open_remote_workspace", RemoteWorkspacePolicy::RemoteRouted),
    ("open_workspace", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "page_create_open_link",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("page_delete", RemoteWorkspacePolicy::WorkspaceAgnostic),
    (
        "page_delete_version",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("page_deploy", RemoteWorkspacePolicy::WorkspaceAgnostic),
    ("page_list", RemoteWorkspacePolicy::WorkspaceAgnostic),
    (
        "page_list_versions",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("page_publish", RemoteWorkspacePolicy::LocalOnly),
    ("page_save_version", RemoteWorkspacePolicy::LocalOnly),
    ("page_unpublish", RemoteWorkspacePolicy::WorkspaceAgnostic),
    ("page_update", RemoteWorkspacePolicy::WorkspaceAgnostic),
    ("paste_files", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "peer_control_attach",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "peer_control_detach",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "peer_controller_set_active",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "peer_host_invoke_complete",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("peer_mode_ping", RemoteWorkspacePolicy::WorkspaceAgnostic),
    (
        "plan_external_mcp_import_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "predownload_acp_client_adapter",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "preview_commit_message",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "probe_acp_client_requirements",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "quick_commit_message",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("quit_app", RemoteWorkspacePolicy::LocalOnly),
    (
        "read_background_command_output",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("read_file_content", RemoteWorkspacePolicy::LegacyUnaudited),
    ("read_mcp_resource", RemoteWorkspacePolicy::LegacyUnaudited),
    ("record_file_change", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "refresh_model_client",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "refresh_models_dev_catalog_now",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "reveal_models_dev_cache_directory",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "refresh_subscription_account",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    ("reject_file", RemoteWorkspacePolicy::LegacyUnaudited),
    ("reject_operation", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "respond_permission",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "respond_permission_batch",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "remove_project_permission_grant",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "recover_interrupted_dialog_turn",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "clear_project_permission_grants",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("reload_config", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "reload_custom_agents",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "reload_global_config",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "reload_session_context",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    ("reload_subagents", RemoteWorkspacePolicy::LegacyUnaudited),
    // One-click self-hosted relay (SSH to user host). WorkspaceAgnostic: uses
    // an SSH connection id, not the open project workspace. See
    // src/web-ui/src/features/relay-deploy/README.md.
    (
        "relay_deploy_cancel",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "relay_deploy_install_docker",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "relay_deploy_poll",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "relay_deploy_preflight",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "relay_deploy_register",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "relay_deploy_start",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "relay_deploy_verify",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "remote_close_workspace",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    (
        "remote_connect_configure_bot",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "remote_connect_configure_custom_server",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "remote_connect_get_bot_verbose_mode",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "remote_connect_get_device_info",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "remote_connect_get_form_state",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "remote_connect_get_lan_ip",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "remote_connect_get_lan_network_info",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "remote_connect_get_methods",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "remote_connect_set_bot_verbose_mode",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "remote_connect_set_form_state",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "remote_connect_start",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "remote_connect_status",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "remote_connect_stop",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "remote_connect_stop_bot",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "remote_connect_weixin_qr_poll",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "remote_connect_weixin_qr_start",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("remote_create_dir", RemoteWorkspacePolicy::RemoteRouted),
    (
        "remote_download_to_local_path",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    ("remote_execute", RemoteWorkspacePolicy::RemoteRouted),
    ("remote_exists", RemoteWorkspacePolicy::RemoteRouted),
    ("remote_get_tree", RemoteWorkspacePolicy::RemoteRouted),
    (
        "remote_get_workspace_info",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    ("remote_open_workspace", RemoteWorkspacePolicy::RemoteRouted),
    ("remote_read_dir", RemoteWorkspacePolicy::RemoteRouted),
    ("remote_read_file", RemoteWorkspacePolicy::RemoteRouted),
    ("remote_remove", RemoteWorkspacePolicy::RemoteRouted),
    (
        "remote_remove_workspace",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    ("remote_rename", RemoteWorkspacePolicy::RemoteRouted),
    (
        "remote_upload_from_local_path",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    ("remote_write_file", RemoteWorkspacePolicy::RemoteRouted),
    (
        "remove_recent_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("rename_file", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "reorder_opened_workspaces",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "replace_mode_skill_selection",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "report_canvas_runtime_error",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "report_ide_control_result",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "reset_agent_profile_config",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "reset_assistant_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("reset_config", RemoteWorkspacePolicy::LegacyUnaudited),
    ("reset_memory", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "reset_mode_skill_selection",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "reset_workspace_persona_files",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "resize_agent_companion_desktop_pet",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    ("restart_app", RemoteWorkspacePolicy::LocalOnly),
    ("restart_mcp_server", RemoteWorkspacePolicy::LegacyUnaudited),
    ("restore_session", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "restore_session_view",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "restore_session_with_turns",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("reveal_in_explorer", RemoteWorkspacePolicy::LocalOnly),
    (
        "review_platform_clear_auth_token",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "review_platform_get_issue",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    (
        "review_platform_get_pull_request_ci_log",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    (
        "review_platform_get_pull_request_detail",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    (
        "review_platform_get_pull_request_detail_page",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    (
        "review_platform_get_pull_request_review_target",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    (
        "review_platform_get_pull_request_review_target_by_identity",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    (
        "review_platform_get_workspace_context",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    (
        "review_platform_get_workspace_snapshot",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    (
        "review_platform_update_auth_token",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("rollback_miniapp", RemoteWorkspacePolicy::LegacyUnaudited),
    ("rollback_session", RemoteWorkspacePolicy::RemoteUnsupported),
    (
        "rollback_session_to_turn",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    ("run_init_agents_md", RemoteWorkspacePolicy::LegacyUnaudited),
    ("run_system_command", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "save_acp_json_config",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("save_canvas_state", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "save_cloud_speech_config",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "save_git_repo_history",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "save_mcp_json_config",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "save_merged_diff_content",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "save_session_metadata",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("save_session_turn", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "scan_workspace_info",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("search_build_index", RemoteWorkspacePolicy::RemoteRouted),
    (
        "search_file_contents",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("search_filenames", RemoteWorkspacePolicy::RemoteRouted),
    ("search_files", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "search_referenceable_sessions",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "search_get_repo_status",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    ("search_rebuild_index", RemoteWorkspacePolicy::RemoteRouted),
    (
        "search_skill_market",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "send_background_command_input",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "send_mcp_app_message",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("send_system_notification", RemoteWorkspacePolicy::LocalOnly),
    (
        "set_acp_session_config_option",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    (
        "set_acp_session_model",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "set_active_workspace",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "set_primary_assistant_workspace",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "set_agent_profile_config",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("set_config", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "set_external_mcp_server_decision_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "set_external_mcp_servers_enabled_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "set_external_source_conflict_choice_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "set_native_prompt_command_conflict_choice_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "set_external_source_enabled_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "set_external_tool_conflict_choice_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "set_external_tool_target_decision_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "set_external_tool_targets_enabled_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "set_external_subagent_activation_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "set_external_subagents_enabled_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "set_external_subagent_model_binding_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "set_global_skill_disabled",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("set_macos_edit_menu_mode", RemoteWorkspacePolicy::LocalOnly),
    (
        "set_main_window_transient_geometry",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "set_prevent_sleep_enabled",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "set_miniapp_draft_storage",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "set_miniapp_storage",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "set_mode_skill_disabled",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "set_session_memory_mode",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "set_session_thread_goal_status",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "set_subagent_timeout",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "show_agent_companion_desktop_pet",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    ("show_main_window", RemoteWorkspacePolicy::LocalOnly),
    (
        "speech_append_audio_chunk",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "speech_cancel_input_session",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "speech_cancel_model_download",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    ("speech_delete_model", RemoteWorkspacePolicy::LocalOnly),
    ("speech_download_model", RemoteWorkspacePolicy::LocalOnly),
    (
        "speech_finish_input_session",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    ("speech_list_models", RemoteWorkspacePolicy::LocalOnly),
    (
        "speech_start_input_session",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    ("speech_verify_model", RemoteWorkspacePolicy::LocalOnly),
    ("ssh_connect", RemoteWorkspacePolicy::WorkspaceAgnostic),
    (
        "ssh_delete_connection",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("ssh_disconnect", RemoteWorkspacePolicy::WorkspaceAgnostic),
    (
        "ssh_disconnect_all",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("ssh_get_config", RemoteWorkspacePolicy::WorkspaceAgnostic),
    (
        "ssh_get_server_info",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "ssh_has_stored_password",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("ssh_is_connected", RemoteWorkspacePolicy::WorkspaceAgnostic),
    (
        "ssh_list_config_hosts",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "ssh_list_docker_containers",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "ssh_list_saved_connections",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "ssh_save_connection",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "ssh_test_connection",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "start_acp_dialog_turn",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("start_dialog_turn", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "subscribe_permission_requests",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("start_file_watch", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "start_mcp_remote_oauth",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("start_mcp_server", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "start_search_file_contents_stream",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "start_search_filenames_stream",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    ("start_subscription_login", RemoteWorkspacePolicy::LocalOnly),
    ("startup_window_control", RemoteWorkspacePolicy::LocalOnly),
    ("steer_dialog_turn", RemoteWorkspacePolicy::LegacyUnaudited),
    ("stop_acp_client", RemoteWorkspacePolicy::LegacyUnaudited),
    ("stop_file_watch", RemoteWorkspacePolicy::LegacyUnaudited),
    ("stop_mcp_server", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "submit_acp_permission_response",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "submit_mcp_interaction_response",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "submit_user_answers",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "subscribe_config_updates",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "sync_config_to_global",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("terminal_ack", RemoteWorkspacePolicy::RemoteRouted),
    ("terminal_close", RemoteWorkspacePolicy::RemoteRouted),
    ("terminal_create", RemoteWorkspacePolicy::RemoteRouted),
    ("terminal_execute", RemoteWorkspacePolicy::RemoteRouted),
    ("terminal_get", RemoteWorkspacePolicy::LegacyUnaudited),
    ("terminal_get_history", RemoteWorkspacePolicy::RemoteRouted),
    (
        "terminal_get_shells",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "terminal_has_shell_integration",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    ("terminal_list", RemoteWorkspacePolicy::LegacyUnaudited),
    ("terminal_resize", RemoteWorkspacePolicy::RemoteRouted),
    ("terminal_send_command", RemoteWorkspacePolicy::RemoteRouted),
    (
        "terminal_shutdown_all",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("terminal_signal", RemoteWorkspacePolicy::RemoteRouted),
    ("terminal_write", RemoteWorkspacePolicy::RemoteRouted),
    (
        "test_ai_config_connection",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("test_ai_connection", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "toggle_main_window_fullscreen",
        RemoteWorkspacePolicy::LocalOnly,
    ),
    (
        "touch_session_activity",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "trigger_announcement",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    ("unarchive_session", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "update_app_status",
        RemoteWorkspacePolicy::WorkspaceAgnostic,
    ),
    (
        "update_active_turn_permission_mode",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    ("update_cron_job", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "update_custom_agent",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "update_external_integration_policy_command",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    (
        "update_mcp_remote_auth",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("update_miniapp", RemoteWorkspacePolicy::LegacyUnaudited),
    ("update_session_mode", RemoteWorkspacePolicy::RemoteRouted),
    (
        "update_session_permission_mode",
        RemoteWorkspacePolicy::RemoteRouted,
    ),
    (
        "update_session_model",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "update_session_thread_goal_objective",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "update_session_title",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("update_subagent", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "update_subagent_config",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "update_workspace_info",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "upload_image_contexts",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    ("validate_config", RemoteWorkspacePolicy::LegacyUnaudited),
    (
        "validate_skill_path",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "validate_tool_input",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "webdriver_bridge_result",
        RemoteWorkspacePolicy::LegacyUnaudited,
    ),
    (
        "worktree_bind_session",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    ("worktree_create", RemoteWorkspacePolicy::RemoteUnsupported),
    (
        "worktree_create_branch",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    ("worktree_list", RemoteWorkspacePolicy::RemoteUnsupported),
    ("worktree_list_projects", RemoteWorkspacePolicy::LocalOnly),
    ("worktree_promote", RemoteWorkspacePolicy::RemoteUnsupported),
    (
        "worktree_recreate",
        RemoteWorkspacePolicy::RemoteUnsupported,
    ),
    ("worktree_remove", RemoteWorkspacePolicy::RemoteUnsupported),
    ("write_file_content", RemoteWorkspacePolicy::LegacyUnaudited),
];

/// The declared remote-workspace stance of a registered Tauri command.
pub fn remote_workspace_policy(command: &str) -> Option<RemoteWorkspacePolicy> {
    openbitfun_product_domains::remote_surface::operation(command).map(|op| op.remote_workspace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use openbitfun_product_domains::remote_surface::{operations, OperationSurface};
    use std::collections::BTreeSet;

    /// Extracts the command names registered in `tauri::generate_handler!`.
    pub(crate) fn registered_commands() -> BTreeSet<String> {
        let source = include_str!("../lib.rs");
        let start = source
            .find("generate_handler![")
            .expect("lib.rs must register commands via tauri::generate_handler!")
            + "generate_handler![".len();
        let block = &source[start..];
        let end = block
            .find("])")
            .expect("generate_handler! block must terminate with `])`");
        block[..end]
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .map(|line| {
                let entry = line.trim_end_matches(',');
                entry
                    .rsplit("::")
                    .next()
                    .expect("command path segments are non-empty")
                    .to_string()
            })
            .collect()
    }

    #[test]
    fn every_registered_command_has_exactly_one_registry_row() {
        let registered = registered_commands();
        assert!(
            registered.len() > 400,
            "generate_handler! parsing looks broken; only {} commands found",
            registered.len()
        );

        let declared: BTreeSet<String> = operations()
            .iter()
            .filter(|op| op.surface == OperationSurface::TauriCommand)
            .map(|op| op.id.to_string())
            .collect();

        let missing: Vec<_> = registered.difference(&declared).cloned().collect();
        assert!(
            missing.is_empty(),
            "commands registered in generate_handler! without a Product Operation Registry row \
             (add one `op(...)` row in src/crates/contracts/product-domains/src/remote_surface/table.rs; \
             new commands must not use Unaudited): {missing:?}"
        );

        let stale: Vec<_> = declared.difference(&registered).cloned().collect();
        assert!(
            stale.is_empty(),
            "registry rows declared as Tauri commands that are no longer registered \
             (delete the row, or mark it HostInvokeOnly if a peer alias must survive): {stale:?}"
        );
    }

    #[test]
    fn token_usage_statistics_are_scoped_to_the_current_bitfun_host() {
        assert_eq!(
            remote_workspace_policy("get_token_usage_statistics"),
            Some(RemoteWorkspacePolicy::WorkspaceAgnostic),
            "token usage is recorded by the current BitFun runtime and does not follow the workspace filesystem to an SSH host"
        );
    }

    #[test]
    fn external_mcp_import_commands_explicitly_reject_remote_workspaces() {
        for command in [
            "plan_external_mcp_import_command",
            "apply_external_mcp_import_command",
        ] {
            assert_eq!(
                remote_workspace_policy(command),
                Some(RemoteWorkspacePolicy::Unsupported),
                "{command} must never fall back to the controller's local MCP config"
            );
        }
    }

    #[test]
    fn workspace_reference_snapshot_explicitly_rejects_remote_workspaces() {
        assert_eq!(
            remote_workspace_policy("get_workspace_reference_snapshot"),
            Some(RemoteWorkspacePolicy::Unsupported),
            "workspace references must never scan controller-local OpenCode config for a remote workspace"
        );
    }

    #[test]
    fn external_hook_import_commands_explicitly_reject_remote_workspaces() {
        for command in [
            "get_external_hook_import_snapshot",
            "plan_external_hook_import_command",
            "apply_external_hook_import_command",
            "mutate_external_hook_import_command",
        ] {
            assert_eq!(
                remote_workspace_policy(command),
                Some(RemoteWorkspacePolicy::Unsupported),
                "{command} must never use local imported Hooks for a remote workspace"
            );
        }
    }

    #[test]
    fn complete_rollback_commands_explicitly_reject_remote_workspaces() {
        for command in ["rollback_session", "rollback_session_to_turn"] {
            assert_eq!(
                remote_workspace_policy(command),
                Some(RemoteWorkspacePolicy::Unsupported),
                "{command} must not offer message-only rollback without remote file snapshots"
            );
        }
    }

    #[test]
    fn external_source_control_command_is_registered() {
        const COMMAND: &str = "get_external_source_control_snapshot";
        assert!(
            registered_commands().contains(COMMAND),
            "Desktop must register the external-source control command"
        );
    }
}
