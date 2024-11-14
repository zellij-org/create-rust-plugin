use zellij_tile::prelude::*;
use std::collections::BTreeMap;

#[derive(Default, Debug)]
pub struct SequentialCommandsToRun {
    open_terminal_pane_ids: Vec<u32>,
    sequence_errored: bool,
    all_commands: Vec<CommandToRun>,
    pending_commands: Vec<CommandToRun>,
    running_command: Option<(CommandToRun, PaneId, usize)>, // usize -> own_id
    next_id: usize,
}

impl SequentialCommandsToRun {
    pub fn set_up_commands(&mut self, commands: Vec<CommandToRun>) {
        self.all_commands = commands.clone();
        self.pending_commands = commands;
        self.running_command = None;
        self.next_id = 0;
    }
    pub fn run_next_commnand(&mut self) {
        if self.sequence_errored {
            self.restart_sequence();
        }
        if self.pending_commands.len() > 0 {
            let command_id = self.next_command_id();
            if let Some(last_command) = self.pending_commands.iter().next() {
                let mut context = BTreeMap::new();
                context.insert("id".to_owned(), format!("{command_id}"));
                open_command_pane_floating(last_command.clone(), None, context);
            }
        }
    }
    pub fn next_command_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    pub fn sequence_is_running(&self) -> bool {
        if self.sequence_errored {
            return false; // TODO: check, handle, etc.
        } else {
            self.pending_commands.len() > 0 || self.running_command.is_some()
        }
    }
    pub fn register_command_opened(&mut self, terminal_pane_id: u32, context: BTreeMap<String, String>) {
        self.open_terminal_pane_ids.push(terminal_pane_id);
        if let Some(command_id) = context.get("id").and_then(|i| i.trim().parse::<usize>().ok()) {
            if command_id == self.next_id.saturating_sub(1) && self.pending_commands.len() > 0 {
                let next_command = self.pending_commands.remove(0);
                self.running_command = Some((next_command, PaneId::Terminal(terminal_pane_id), command_id));
            }
        } else {
            eprintln!("Cannot find command id");
        }
    }
    pub fn register_command_exited(&mut self, terminal_pane_id: u32, exit_code: Option<i32>, context: BTreeMap<String, String>) {
        if exit_code == Some(0) {
            if let Some(command_id) = context.get("id").and_then(|i| i.trim().parse::<usize>().ok()) {
                if command_id == self.next_id.saturating_sub(1) {
                    self.running_command = None;
                    close_terminal_pane(terminal_pane_id);
                    self.run_next_commnand();
                }
            } else {
                eprintln!("Cannot find command id");
            }
        } else {
            self.sequence_errored = true;
        }
    }
    pub fn sequence_finished(&self) -> bool {
        self.pending_commands.is_empty() && self.running_command.is_none()
    }
    fn restart_sequence(&mut self) {
        for terminal_pane_id in self.open_terminal_pane_ids.drain(..) {
            close_terminal_pane(terminal_pane_id);
        }
        self.pending_commands = self.all_commands.clone();
        self.running_command = None;
        self.sequence_errored = false;
    }
}
