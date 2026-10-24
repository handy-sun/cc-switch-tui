use crate::app_config::{AppType, SkillApps};
use crate::cli::i18n::texts;
use crate::error::AppError;
use crate::services::SkillService;

use super::super::app::{LoadingKind, Overlay, ToastKind};
use super::super::route::Route;
use super::super::runtime_skills::{
    finish_skills_import_with, open_agent_skills_import_picker, open_skills_import_picker,
    parse_repo_spec,
};
use super::RuntimeActionContext;

/// external 模式下写操作被拦截：转成提示 toast，而不是让 TUI 弹出错误。
fn blocked_by_external_mode(ctx: &mut RuntimeActionContext<'_>, error: &AppError) -> bool {
    if !crate::services::SkillService::is_external_mode() {
        return false;
    }
    ctx.app
        .push_toast(texts::tui_toast_skills_write_blocked(), ToastKind::Warning);
    log::debug!("[skills] write blocked in external mode: {error}");
    true
}

pub(super) fn toggle(
    ctx: &mut RuntimeActionContext<'_>,
    directory: String,
    enabled: bool,
) -> Result<(), AppError> {
    if let Err(error) = SkillService::toggle_app(&directory, &ctx.app.app_type, enabled) {
        if blocked_by_external_mode(ctx, &error) {
            return Ok(());
        }
        return Err(error);
    }
    *ctx.data = super::super::data::UiData::load(&ctx.app.app_type)?;
    ctx.app.push_toast(
        texts::tui_toast_skill_toggled(&directory, enabled),
        ToastKind::Success,
    );
    Ok(())
}

pub(super) fn toggle_many(
    ctx: &mut RuntimeActionContext<'_>,
    directories: Vec<String>,
    enabled: bool,
) -> Result<(), AppError> {
    for directory in &directories {
        if let Err(error) = SkillService::toggle_app(directory, &ctx.app.app_type, enabled) {
            if blocked_by_external_mode(ctx, &error) {
                return Ok(());
            }
            return Err(error);
        }
    }
    *ctx.data = super::super::data::UiData::load(&ctx.app.app_type)?;
    ctx.app.push_toast(
        texts::tui_toast_skills_toggled(directories.len(), enabled),
        ToastKind::Success,
    );
    Ok(())
}

pub(super) fn set_apps(
    ctx: &mut RuntimeActionContext<'_>,
    directory: String,
    apps: SkillApps,
) -> Result<(), AppError> {
    let Some(before) = ctx
        .data
        .skills
        .installed
        .iter()
        .find(|skill| skill.directory == directory)
        .map(|skill| skill.apps.clone())
    else {
        ctx.app
            .push_toast(texts::tui_skill_not_found(), ToastKind::Warning);
        return Ok(());
    };

    let mut changed = false;
    for app_type in AppType::all() {
        let next_enabled = apps.is_enabled_for(&app_type);
        if before.is_enabled_for(&app_type) == next_enabled {
            continue;
        }
        changed = true;
        if let Err(error) = SkillService::toggle_app(&directory, &app_type, next_enabled) {
            if blocked_by_external_mode(ctx, &error) {
                return Ok(());
            }
            return Err(error);
        }
    }

    *ctx.data = super::super::data::UiData::load(&ctx.app.app_type)?;
    if changed {
        ctx.app
            .push_toast(texts::tui_toast_skill_apps_updated(), ToastKind::Success);
    }
    Ok(())
}

pub(super) fn set_apps_many(
    ctx: &mut RuntimeActionContext<'_>,
    directories: Vec<String>,
    apps: SkillApps,
) -> Result<(), AppError> {
    let mut changed = false;
    for directory in &directories {
        let Some(before) = ctx
            .data
            .skills
            .installed
            .iter()
            .find(|skill| skill.directory == *directory)
            .map(|skill| skill.apps.clone())
        else {
            continue;
        };

        for app_type in AppType::all() {
            let next_enabled = apps.is_enabled_for(&app_type);
            if before.is_enabled_for(&app_type) == next_enabled {
                continue;
            }
            changed = true;
            if let Err(error) = SkillService::toggle_app(directory, &app_type, next_enabled) {
                if blocked_by_external_mode(ctx, &error) {
                    return Ok(());
                }
                return Err(error);
            }
        }
    }

    *ctx.data = super::super::data::UiData::load(&ctx.app.app_type)?;
    if changed {
        ctx.app
            .push_toast(texts::tui_toast_skill_apps_updated(), ToastKind::Success);
    }
    Ok(())
}

pub(super) fn install(ctx: &mut RuntimeActionContext<'_>, spec: String) -> Result<(), AppError> {
    if let Err(error) = SkillService::ensure_managed(crate::services::skill::SkillsWriteOp::Install)
    {
        if blocked_by_external_mode(ctx, &error) {
            return Ok(());
        }
        return Err(error);
    }
    let Some(tx) = ctx.skills_req_tx else {
        return Err(AppError::Message(
            texts::tui_error_skills_worker_unavailable().to_string(),
        ));
    };
    ctx.app.overlay = Overlay::Loading {
        kind: LoadingKind::Generic,
        title: texts::tui_skills_install_title().to_string(),
        message: texts::tui_loading().to_string(),
    };
    tx.send(super::super::runtime_systems::SkillsReq::Install {
        spec: spec.clone(),
        app: ctx.app.app_type.clone(),
    })
    .map_err(|e| AppError::Message(e.to_string()))?;
    Ok(())
}

pub(super) fn uninstall(
    ctx: &mut RuntimeActionContext<'_>,
    directory: String,
) -> Result<(), AppError> {
    if let Err(error) = SkillService::uninstall(&directory) {
        if blocked_by_external_mode(ctx, &error) {
            return Ok(());
        }
        return Err(error);
    }
    *ctx.data = super::super::data::UiData::load(&ctx.app.app_type)?;
    ctx.app.push_toast(
        texts::tui_toast_skill_uninstalled(&directory),
        ToastKind::Success,
    );
    if matches!(&ctx.app.route, Route::SkillDetail { directory: current } if current.eq_ignore_ascii_case(&directory))
    {
        if matches!(ctx.app.route_stack.last(), Some(Route::Skills)) {
            ctx.app.route_stack.pop();
        }
        ctx.app.route = Route::Skills;
    }
    Ok(())
}

pub(super) fn uninstall_many(
    ctx: &mut RuntimeActionContext<'_>,
    directories: Vec<String>,
) -> Result<(), AppError> {
    for directory in &directories {
        if let Err(error) = SkillService::uninstall(directory) {
            if blocked_by_external_mode(ctx, &error) {
                return Ok(());
            }
            return Err(error);
        }
    }
    *ctx.data = super::super::data::UiData::load(&ctx.app.app_type)?;
    ctx.app.push_toast(
        texts::tui_toast_skills_uninstalled(directories.len()),
        ToastKind::Success,
    );
    Ok(())
}

pub(super) fn sync(
    ctx: &mut RuntimeActionContext<'_>,
    scope: Option<AppType>,
) -> Result<(), AppError> {
    if let Err(error) = SkillService::sync_all_enabled(scope.as_ref()) {
        if blocked_by_external_mode(ctx, &error) {
            return Ok(());
        }
        return Err(error);
    }
    *ctx.data = super::super::data::UiData::load(&ctx.app.app_type)?;
    ctx.app
        .push_toast(texts::tui_toast_skills_synced(), ToastKind::Success);
    Ok(())
}

pub(super) fn discover(ctx: &mut RuntimeActionContext<'_>, query: String) -> Result<(), AppError> {
    let Some(tx) = ctx.skills_req_tx else {
        return Err(AppError::Message(
            texts::tui_error_skills_worker_unavailable().to_string(),
        ));
    };
    ctx.app.overlay = Overlay::Loading {
        kind: LoadingKind::Generic,
        title: texts::tui_skills_discover_title().to_string(),
        message: texts::tui_loading().to_string(),
    };
    tx.send(super::super::runtime_systems::SkillsReq::Discover { query })
        .map_err(|e| AppError::Message(e.to_string()))?;
    Ok(())
}

pub(super) fn repo_add(ctx: &mut RuntimeActionContext<'_>, spec: String) -> Result<(), AppError> {
    let repo = parse_repo_spec(&spec)?;
    SkillService::upsert_repo(repo)?;
    *ctx.data = super::super::data::UiData::load(&ctx.app.app_type)?;
    ctx.app
        .push_toast(texts::tui_toast_repo_added(), ToastKind::Success);
    Ok(())
}

pub(super) fn repo_remove(
    ctx: &mut RuntimeActionContext<'_>,
    owner: String,
    name: String,
) -> Result<(), AppError> {
    SkillService::remove_repo(&owner, &name)?;
    *ctx.data = super::super::data::UiData::load(&ctx.app.app_type)?;
    ctx.app
        .push_toast(texts::tui_toast_repo_removed(), ToastKind::Success);
    Ok(())
}

pub(super) fn repo_toggle_enabled(
    ctx: &mut RuntimeActionContext<'_>,
    owner: String,
    name: String,
    enabled: bool,
) -> Result<(), AppError> {
    let mut index = SkillService::load_index()?;
    if let Some(repo) = index
        .repos
        .iter_mut()
        .find(|r| r.owner == owner && r.name == name)
    {
        repo.enabled = enabled;
        SkillService::save_index(&index)?;
    }
    *ctx.data = super::super::data::UiData::load(&ctx.app.app_type)?;
    ctx.app
        .push_toast(texts::tui_toast_repo_toggled(enabled), ToastKind::Success);
    Ok(())
}

pub(super) fn open_import(ctx: &mut RuntimeActionContext<'_>) -> Result<(), AppError> {
    open_skills_import_picker(ctx.app)
}

pub(super) fn open_agent_import(ctx: &mut RuntimeActionContext<'_>) -> Result<(), AppError> {
    open_agent_skills_import_picker(ctx.app)
}

pub(super) fn import_from_apps(
    ctx: &mut RuntimeActionContext<'_>,
    directories: Vec<String>,
) -> Result<(), AppError> {
    if let Err(error) =
        SkillService::ensure_managed(crate::services::skill::SkillsWriteOp::ImportFromApps)
    {
        if blocked_by_external_mode(ctx, &error) {
            return Ok(());
        }
        return Err(error);
    }
    if let Err(error) = finish_skills_import_with(
        ctx.app,
        ctx.data,
        || SkillService::import_from_apps(directories),
        super::super::data::UiData::load,
    ) {
        if blocked_by_external_mode(ctx, &error) {
            return Ok(());
        }
        return Err(error);
    }
    ctx.app.skills_unmanaged_results = SkillService::scan_unmanaged()?;
    ctx.app.skills_unmanaged_selected.clear();
    ctx.app.skills_unmanaged_idx = 0;
    Ok(())
}

pub(super) fn import_from_agent(
    ctx: &mut RuntimeActionContext<'_>,
    directories: Vec<String>,
) -> Result<(), AppError> {
    if let Err(error) =
        SkillService::ensure_managed(crate::services::skill::SkillsWriteOp::ImportFromAgent)
    {
        if blocked_by_external_mode(ctx, &error) {
            return Ok(());
        }
        return Err(error);
    }
    if let Err(error) = finish_skills_import_with(
        ctx.app,
        ctx.data,
        || SkillService::import_from_agent(directories),
        super::super::data::UiData::load,
    ) {
        if blocked_by_external_mode(ctx, &error) {
            return Ok(());
        }
        return Err(error);
    }
    ctx.app.skills_unmanaged_results = SkillService::scan_agent_installed()?;
    ctx.app.skills_unmanaged_selected.clear();
    ctx.app.skills_unmanaged_idx = 0;
    Ok(())
}
