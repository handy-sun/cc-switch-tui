use super::is_chinese;

pub fn tui_settings_skills_mode_label() -> &'static str {
    if is_chinese() {
        "技能管理模式"
    } else {
        "Skills Mode"
    }
}

pub fn tui_skills_mode_name(mode: crate::settings::SkillsMode) -> &'static str {
    match mode {
        crate::settings::SkillsMode::Managed => {
            if is_chinese() {
                "本工具管理"
            } else {
                "Managed by cc-switch-tui"
            }
        }
        crate::settings::SkillsMode::External => {
            if is_chinese() {
                "外部管理（agent-skills-nix）"
            } else {
                "External (agent-skills-nix)"
            }
        }
    }
}

pub fn tui_skills_external_banner() -> &'static str {
    if is_chinese() {
        "外部管理：Skills 由 agent-skills-nix 声明，本页面只读"
    } else {
        "External mode: skills are declared by agent-skills-nix; this page is read-only"
    }
}

pub fn tui_skills_external_hint() -> String {
    crate::services::skill::external_mode_message()
}

pub fn tui_confirm_skills_mode_external() -> &'static str {
    if is_chinese() {
        "切换到外部管理后，本工具不再安装、启用、同步或卸载任何 Skill，也不再打包或恢复 WebDAV 的 skills.zip。请先在 expnix/skills-nix 中确认接管范围。确认切换？"
    } else {
        "In external mode this tool stops installing, enabling, syncing and uninstalling skills, and no longer packs or restores WebDAV skills.zip. Confirm that agent-skills-nix already owns the target directories. Switch now?"
    }
}

pub fn tui_confirm_skills_mode_managed() -> &'static str {
    if is_chinese() {
        "切回本工具管理前，请先停止 agent-skills-nix 对目标目录的接管并确认备份完整，否则两个写入者会竞争同一批目录。确认切换？"
    } else {
        "Before switching back, disable agent-skills-nix targets and confirm your backups: otherwise two writers will fight over the same directories. Switch now?"
    }
}

pub fn tui_toast_skills_mode_set(mode: crate::settings::SkillsMode) -> String {
    if is_chinese() {
        format!("技能管理模式已设置为：{}。", tui_skills_mode_name(mode))
    } else {
        format!("Skills mode set to: {}.", tui_skills_mode_name(mode))
    }
}

pub fn tui_toast_skills_write_blocked() -> String {
    tui_skills_external_hint()
}
