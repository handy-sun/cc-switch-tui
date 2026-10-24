use super::*;

impl App {
    pub(crate) fn skills_visual_range(&self, len: usize) -> Option<(usize, usize)> {
        let anchor = self.skills_visual_anchor?;
        if len == 0 {
            return None;
        }
        let current = self.skills_idx.min(len - 1);
        let anchor = anchor.min(len - 1);
        Some((anchor.min(current), anchor.max(current)))
    }

    fn selected_installed_skill_indices(&self, len: usize) -> Vec<usize> {
        if len == 0 {
            return Vec::new();
        }

        if let Some((start, end)) = self.skills_visual_range(len) {
            return (start..=end).collect();
        }

        vec![self.skills_idx.min(len - 1)]
    }

    fn selected_installed_skill_directories(
        &self,
        visible: &[&crate::services::skill::InstalledSkill],
    ) -> Vec<String> {
        self.selected_installed_skill_indices(visible.len())
            .into_iter()
            .filter_map(|idx| visible.get(idx))
            .map(|skill| skill.directory.clone())
            .collect()
    }

    pub(crate) fn main_proxy_action(&self, data: &UiData) -> Action {
        let Some(current_app_routed) = data.proxy.routes_current_app_through_proxy(&self.app_type)
        else {
            return Action::None;
        };

        if data.proxy.running && !data.proxy.managed_runtime && !current_app_routed {
            return Action::None;
        }

        Action::SetManagedProxyForCurrentApp {
            app_type: self.app_type.clone(),
            enabled: !current_app_routed,
        }
    }

    pub(crate) fn on_skills_installed_key(&mut self, key: KeyEvent, data: &UiData) -> Action {
        let visible = visible_skills_installed(&self.filter, data);

        // external 模式下先拦写操作快捷键：不让用户走进确认弹窗后发现操作被拒。
        if crate::services::SkillService::is_external_mode()
            && matches!(
                key.code,
                KeyCode::Char('x')
                    | KeyCode::Char(' ')
                    | KeyCode::Char('m')
                    | KeyCode::Char('d')
                    | KeyCode::Char('i')
                    | KeyCode::Char('s')
            )
        {
            self.skills_pending_g = false;
            self.push_toast(texts::tui_skills_external_hint(), ToastKind::Warning);
            return Action::None;
        }

        match key.code {
            KeyCode::Up => {
                self.skills_pending_g = false;
                self.skills_idx = self.skills_idx.saturating_sub(1);
                Action::None
            }
            KeyCode::Down => {
                self.skills_pending_g = false;
                if !visible.is_empty() {
                    self.skills_idx = (self.skills_idx + 1).min(visible.len() - 1);
                }
                Action::None
            }
            KeyCode::Char('g') => {
                if self.skills_pending_g {
                    self.skills_idx = 0;
                    self.skills_pending_g = false;
                } else {
                    self.skills_pending_g = true;
                }
                Action::None
            }
            KeyCode::Char('G') => {
                self.skills_pending_g = false;
                if !visible.is_empty() {
                    self.skills_idx = visible.len() - 1;
                }
                Action::None
            }
            KeyCode::Char('v') => {
                self.skills_pending_g = false;
                if visible.is_empty() {
                    self.skills_visual_anchor = None;
                } else if self.skills_visual_anchor.is_some() {
                    self.skills_visual_anchor = None;
                } else {
                    self.skills_visual_anchor = Some(self.skills_idx.min(visible.len() - 1));
                }
                Action::None
            }
            KeyCode::Enter => {
                self.skills_pending_g = false;
                let Some(skill) = visible.get(self.skills_idx) else {
                    return Action::None;
                };
                self.push_route_and_switch(Route::SkillDetail {
                    directory: skill.directory.clone(),
                })
            }
            KeyCode::Char('x') | KeyCode::Char(' ') => {
                self.skills_pending_g = false;
                let Some(skill) = visible.get(self.skills_idx) else {
                    return Action::None;
                };
                let enabled = !skill.apps.is_enabled_for(&self.app_type);
                let directories = self.selected_installed_skill_directories(&visible);
                self.skills_visual_anchor = None;
                if directories.len() > 1 {
                    Action::SkillsToggleMany {
                        directories,
                        enabled,
                    }
                } else {
                    Action::SkillsToggle {
                        directory: skill.directory.clone(),
                        enabled,
                    }
                }
            }
            KeyCode::Char('m') => {
                self.skills_pending_g = false;
                let Some(skill) = visible.get(self.skills_idx) else {
                    return Action::None;
                };
                let directories = self.selected_installed_skill_directories(&visible);
                let name = if directories.len() > 1 {
                    texts::tui_skills_batch_selection_name(directories.len())
                } else {
                    skill.name.clone()
                };
                self.overlay = Overlay::SkillsAppsPicker {
                    directory: skill.directory.clone(),
                    directories,
                    name,
                    selected: four_app_picker_index(&self.app_type),
                    apps: skill.apps.clone(),
                };
                Action::None
            }
            KeyCode::Char('d') => {
                self.skills_pending_g = false;
                let Some(skill) = visible.get(self.skills_idx) else {
                    return Action::None;
                };
                let directories = self.selected_installed_skill_directories(&visible);
                self.overlay = Overlay::Confirm(ConfirmOverlay {
                    title: texts::tui_skills_uninstall_title().to_string(),
                    message: if directories.len() > 1 {
                        texts::tui_confirm_uninstall_skills_message(directories.len())
                    } else {
                        texts::tui_confirm_uninstall_skill_message(&skill.name, &skill.directory)
                    },
                    action: if directories.len() > 1 {
                        ConfirmAction::SkillsUninstallMany { directories }
                    } else {
                        ConfirmAction::SkillsUninstall {
                            directory: skill.directory.clone(),
                        }
                    },
                });
                Action::None
            }
            KeyCode::Char('i') => {
                self.skills_pending_g = false;
                Action::SkillsOpenImport
            }
            KeyCode::Char('s') => {
                self.skills_pending_g = false;
                Action::SkillsOpenAgentImport
            }
            KeyCode::Char('f') => {
                self.skills_pending_g = false;
                self.push_route_and_switch(Route::SkillsDiscover)
            }
            _ => {
                self.skills_pending_g = false;
                Action::None
            }
        }
    }

    pub(crate) fn on_skills_discover_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Up => {
                self.skills_discover_idx = self.skills_discover_idx.saturating_sub(1);
                Action::None
            }
            KeyCode::Down => {
                let visible = visible_skills_discover(&self.filter, &self.skills_discover_results);
                if !visible.is_empty() {
                    self.skills_discover_idx =
                        (self.skills_discover_idx + 1).min(visible.len() - 1);
                }
                Action::None
            }
            KeyCode::Char('f') => {
                self.overlay = Overlay::TextInput(TextInputState {
                    title: texts::tui_skills_discover_title().to_string(),
                    prompt: texts::tui_skills_discover_prompt().to_string(),
                    input: TextInput::new(self.skills_discover_query.clone()),
                    submit: TextSubmit::SkillsDiscoverQuery,
                    secret: false,
                });
                Action::None
            }
            KeyCode::Enter => {
                let visible = visible_skills_discover(&self.filter, &self.skills_discover_results);
                let Some(skill) = visible.get(self.skills_discover_idx) else {
                    return Action::None;
                };
                if skill.installed {
                    self.push_toast(texts::tui_toast_skill_already_installed(), ToastKind::Info);
                    return Action::None;
                }
                if crate::services::SkillService::is_external_mode() {
                    self.push_toast(texts::tui_skills_external_hint(), ToastKind::Warning);
                    return Action::None;
                }
                Action::SkillsInstall {
                    spec: skill.key.clone(),
                }
            }
            KeyCode::Char('r') => self.push_route_and_switch(Route::SkillsRepos),
            _ => Action::None,
        }
    }

    pub(crate) fn on_skills_repos_key(&mut self, key: KeyEvent, data: &UiData) -> Action {
        let visible = visible_skills_repos(&self.filter, data);
        match key.code {
            KeyCode::Up => {
                self.skills_repo_idx = self.skills_repo_idx.saturating_sub(1);
                Action::None
            }
            KeyCode::Down => {
                if !visible.is_empty() {
                    self.skills_repo_idx = (self.skills_repo_idx + 1).min(visible.len() - 1);
                }
                Action::None
            }
            KeyCode::Char('a') => {
                self.overlay = Overlay::TextInput(TextInputState {
                    title: texts::tui_skills_repos_add_title().to_string(),
                    prompt: texts::tui_skills_repos_add_prompt().to_string(),
                    input: TextInput::new(""),
                    submit: TextSubmit::SkillsRepoAdd,
                    secret: false,
                });
                Action::None
            }
            KeyCode::Char('d') => {
                let Some(repo) = visible.get(self.skills_repo_idx) else {
                    return Action::None;
                };
                self.overlay = Overlay::Confirm(ConfirmOverlay {
                    title: texts::tui_skills_repos_remove_title().to_string(),
                    message: texts::tui_confirm_remove_repo_message(&repo.owner, &repo.name),
                    action: ConfirmAction::SkillsRepoRemove {
                        owner: repo.owner.clone(),
                        name: repo.name.clone(),
                    },
                });
                Action::None
            }
            KeyCode::Char('x') | KeyCode::Char(' ') => {
                let Some(repo) = visible.get(self.skills_repo_idx) else {
                    return Action::None;
                };
                Action::SkillsRepoToggleEnabled {
                    owner: repo.owner.clone(),
                    name: repo.name.clone(),
                    enabled: !repo.enabled,
                }
            }
            _ => Action::None,
        }
    }

    pub(crate) fn on_skill_detail_key(
        &mut self,
        key: KeyEvent,
        data: &UiData,
        directory: &str,
    ) -> Action {
        if crate::services::SkillService::is_external_mode()
            && matches!(
                key.code,
                KeyCode::Char('x')
                    | KeyCode::Char(' ')
                    | KeyCode::Char('m')
                    | KeyCode::Char('d')
                    | KeyCode::Char('s')
                    | KeyCode::Char('S')
            )
        {
            self.push_toast(texts::tui_skills_external_hint(), ToastKind::Warning);
            return Action::None;
        }

        let Some(skill) = data
            .skills
            .installed
            .iter()
            .find(|s| s.directory.eq_ignore_ascii_case(directory))
        else {
            return Action::None;
        };

        match key.code {
            KeyCode::Char('x') | KeyCode::Char(' ') => Action::SkillsToggle {
                directory: skill.directory.clone(),
                enabled: !skill.apps.is_enabled_for(&self.app_type),
            },
            KeyCode::Char('m') => {
                self.overlay = Overlay::SkillsAppsPicker {
                    directory: skill.directory.clone(),
                    directories: vec![skill.directory.clone()],
                    name: skill.name.clone(),
                    selected: four_app_picker_index(&self.app_type),
                    apps: skill.apps.clone(),
                };
                Action::None
            }
            KeyCode::Char('d') => {
                self.overlay = Overlay::Confirm(ConfirmOverlay {
                    title: texts::tui_skills_uninstall_title().to_string(),
                    message: texts::tui_confirm_uninstall_skill_message(
                        &skill.name,
                        &skill.directory,
                    ),
                    action: ConfirmAction::SkillsUninstall {
                        directory: skill.directory.clone(),
                    },
                });
                Action::None
            }
            KeyCode::Char('s') => Action::SkillsSync {
                app: Some(self.app_type.clone()),
            },
            KeyCode::Char('S') => Action::SkillsSync { app: None },
            _ => Action::None,
        }
    }
}
