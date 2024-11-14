mod filepicker_requests;
mod sequential_commands_to_run;
mod workspace_layout;

use zellij_tile::prelude::*;

use std::collections::BTreeMap;
use std::path::PathBuf;

use filepicker_requests::FilePickerRequests;
use sequential_commands_to_run::SequentialCommandsToRun;
use workspace_layout::workspace_layout;

#[derive(Default)]
struct State {
    plugin_name: Option<String>,
    parent_folder: Option<PathBuf>,
    destination_folder_exists: bool,
    error: Option<String>,
    filepicker_requests: FilePickerRequests,
    sequential_commands_to_run: SequentialCommandsToRun,
}

register_plugin!(State);

const DEVELOP_RUST_PLUGIN_URL: &str = "https://github.com/zellij-org/develop-rust-plugin/releases/download/v0.1.0/develop-rust-plugin.wasm";
const RUST_PLUGIN_EXAMPLE_URL: &str = "https://github.com/zellij-org/rust-plugin-example.git";

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ChangeApplicationState,
            PermissionType::ReadApplicationState,
            PermissionType::MessageAndLaunchOtherPlugins,
            PermissionType::RunCommands,
        ]);
        subscribe(&[
            EventType::Key,
            EventType::CommandPaneOpened,
            EventType::CommandPaneExited,
            EventType::RunCommandResult,
        ]);
        if let Some(cwd) = configuration.get("project_dir") {
            self.parent_folder = Some(PathBuf::from(cwd));
        } else {
            let plugin_ids = get_plugin_ids();
            self.parent_folder = Some(plugin_ids.initial_cwd);
        }
    }
    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::RunCommandResult(exit_code, _stdout, _stderr, context) => {
                self.handle_folder_lookup_response(exit_code, context);
                should_render = true;
            }
            Event::Key(key) => {
                if self.error.take().is_some() {
                    // clear error before doing anything else
                    return true;
                }
                match key.bare_key {
                    BareKey::Char(character) if key.has_no_modifiers() => {
                        let plugin_name = self.plugin_name.get_or_insert_with(String::new);
                        plugin_name.push(character);
                        self.query_folder_exists();
                        should_render = true;
                    }
                    BareKey::Backspace if key.has_no_modifiers() => {
                        self.plugin_name.as_mut().and_then(|n| n.pop());
                        if self
                            .plugin_name
                            .as_ref()
                            .map(|n| n.is_empty())
                            .unwrap_or(false)
                        {
                            self.plugin_name = None;
                        }
                        self.query_folder_exists();
                        should_render = true;
                    }
                    BareKey::Char('c') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                        self.plugin_name = None;
                        should_render = true;
                    }
                    BareKey::Char('f') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                        self.filepicker_requests.send_filepicker_request();
                    }
                    BareKey::Enter if key.has_no_modifiers() => {
                        if self.plugin_name.is_none() {
                            self.error = Some("Plugin name cannot be empty".to_owned());
                            should_render = true;
                        } else if self.sequential_commands_to_run.sequence_is_running() {
                            self.error = Some("Already running!".to_owned());
                            should_render = true;
                        } else if self.destination_folder_exists {
                            // do not run any commands, only open the workspace
                            self.start_workspace();
                        } else {
                            self.set_up_commands();
                            self.sequential_commands_to_run.run_next_commnand();
                        }
                    }
                    _ => {}
                }
            }
            Event::CommandPaneOpened(terminal_pane_id, context) => {
                self.sequential_commands_to_run
                    .register_command_opened(terminal_pane_id, context);
                should_render = true;
            }
            Event::CommandPaneExited(terminal_pane_id, exit_code, context) => {
                self.sequential_commands_to_run.register_command_exited(
                    terminal_pane_id,
                    exit_code,
                    context,
                );
                if self.sequential_commands_to_run.sequence_finished() {
                    self.start_workspace();
                }
            }
            _ => {}
        }
        should_render
    }
    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        let mut should_render = false;
        if pipe_message.name == "filepicker_result" {
            if let Some(chosen_folder) = self
                .filepicker_requests
                .handle_filepicker_response(pipe_message)
            {
                self.parent_folder = Some(chosen_folder);
                self.query_folder_exists();
                should_render = true;
            }
        }
        should_render
    }
    fn render(&mut self, rows: usize, cols: usize) {
        let plugin_parent_folder_display = self.plugin_parent_folder_display();
        let plugin_name_display = self.plugin_name_display();
        let title_text = "Hi there! Let's create your new Zellij Rust plugin. I'm about to:";
        let bulletin_1 = format!(
            "1. Create a new folder {}/{}",
            plugin_parent_folder_display, plugin_name_display
        );
        let bulletin_2 = format!("2. Git clone and adjust a template repository into it.");
        let bulletin_3 =
            format!("3. Open a new tab with your $EDITOR and a development environment");
        let prompt_text = if let Some(plugin_name) = &self.plugin_name {
            format!("Plugin name: {}_", plugin_name)
        } else {
            format!("Plugin name: _")
        };
        let help_text =
            "Help: <ENTER> - create plugin, <Ctrl f> - change root folder, <ESC> - close";

        let max_width = bulletin_3.chars().count() + 2;
        let x_start_position = cols.saturating_sub(max_width) / 2;
        let y_start_position = rows.saturating_sub(8) / 2; // 8 is the line count

        let title_text = Text::new(title_text).color_range(2, ..);
        let bulletin_1 = Text::new(bulletin_1)
            .color_range(0, 23..=23 + plugin_parent_folder_display.chars().count())
            .color_range(3, 24 + plugin_parent_folder_display.chars().count()..);
        let bulletin_2 = Text::new(bulletin_2);
        let bulletin_3 = Text::new(bulletin_3).color_range(0, 28..=34);
        let prompt_text = Text::new(prompt_text)
            .color_range(2, ..=11)
            .color_range(3, 13..);
        let warning_text = if self.destination_folder_exists {
            Some(
                "Warning: destination folder exists, will start a workspace without overriding it.",
            )
        } else {
            None
        };
        let error_text = self.error.as_ref().map(|error| format!("Error: {}", error));
        let help_text = Text::new(help_text)
            .color_range(3, 6..=12)
            .color_range(3, 31..=38)
            .color_range(3, 62..=66);

        print_text_with_coordinates(title_text, x_start_position, y_start_position, None, None);
        print_text_with_coordinates(
            bulletin_1,
            x_start_position + 2,
            y_start_position + 2,
            None,
            None,
        );
        print_text_with_coordinates(
            bulletin_2,
            x_start_position + 2,
            y_start_position + 3,
            None,
            None,
        );
        print_text_with_coordinates(
            bulletin_3,
            x_start_position + 2,
            y_start_position + 4,
            None,
            None,
        );
        print_text_with_coordinates(
            prompt_text,
            x_start_position,
            y_start_position + 6,
            None,
            None,
        );

        if let Some(warning_text) = warning_text {
            print_text_with_coordinates(
                Text::new(warning_text).color_range(3, ..),
                x_start_position,
                y_start_position + 8,
                None,
                None,
            );
        }
        if let Some(error_text) = error_text {
            let line_index = if warning_text.is_some() {
                y_start_position + 10
            } else {
                y_start_position + 8
            };
            print_text_with_coordinates(
                Text::new(error_text).color_range(3, ..),
                x_start_position,
                line_index,
                None,
                None,
            );
        }
        print_text_with_coordinates(help_text, 0, rows, None, None);
    }
}

impl State {
    fn query_folder_exists(&mut self) {
        if let Some(plugin_name) = &self.plugin_name {
            if let Some(parent_folder) = &self.parent_folder {
                let mut context = BTreeMap::new();
                context.insert("folder_lookup".to_owned(), plugin_name.clone());
                run_command(
                    &[
                        "ls",
                        &format!(
                            "{}/{}/Cargo.toml",
                            parent_folder.display(),
                            plugin_name.clone()
                        ),
                    ],
                    context,
                );
            }
        }
        self.destination_folder_exists = false;
    }
    fn handle_folder_lookup_response(
        &mut self,
        exit_code: Option<i32>,
        context: BTreeMap<String, String>,
    ) {
        if let Some(looked_up_plugin_name) = context.get("folder_lookup") {
            if Some(looked_up_plugin_name) == self.plugin_name.as_ref() {
                if exit_code == Some(0) {
                    self.destination_folder_exists = true;
                }
            }
        }
    }
    fn plugin_parent_folder_display(&self) -> String {
        self.parent_folder
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| format!("<UNKNOWN>"))
    }
    fn plugin_name_display(&self) -> String {
        self.plugin_name
            .clone()
            .unwrap_or_else(|| format!("<YOUR PLUGIN NAME>"))
    }
    fn set_up_commands(&mut self) {
        if let Some(parent_folder) = &self.parent_folder {
            if let Some(plugin_name) = &self.plugin_name {
                let parent_folder_display = parent_folder.display();
                let destination_folder = format!("{parent_folder_display}/{plugin_name}");
                let cargo_toml_path = &format!("{parent_folder_display}/{plugin_name}/Cargo.toml");
                let cargo_lock_path = &format!("{parent_folder_display}/{plugin_name}/Cargo.lock");
                let mut commands = vec![
                    CommandToRun::new_with_args(
                        "git",
                        vec!["clone", RUST_PLUGIN_EXAMPLE_URL, &destination_folder],
                    ),
                    CommandToRun::new_with_args(
                        "sed",
                        vec![
                            "-i",
                            &format!("s/rust-plugin-example/{plugin_name}/g"),
                            cargo_toml_path,
                            cargo_lock_path,
                        ],
                    ),
                ];
                for command in commands.iter_mut() {
                    command.cwd = Some(parent_folder.clone());
                }
                self.sequential_commands_to_run.set_up_commands(commands);
            }
        }
    }
    fn start_workspace(&self) {
        if let Some(parent_folder) = &self.parent_folder {
            if let Some(plugin_name) = &self.plugin_name {
                let plugin_folder = parent_folder.join(plugin_name);
                new_tabs_with_layout_info(LayoutInfo::Stringified(workspace_layout(
                    &plugin_folder,
                    DEVELOP_RUST_PLUGIN_URL,
                )));
                close_self();
            }
        }
    }
}

