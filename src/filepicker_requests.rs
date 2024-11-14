use zellij_tile::prelude::*;
use std::collections::BTreeMap;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct FilePickerRequests {
    request_ids: Vec<String>,
}

impl FilePickerRequests {
    pub fn send_filepicker_request(&mut self) {
        let mut args = BTreeMap::new();
        let request_id = Uuid::new_v4();
        self.request_ids.push(request_id.to_string());
        let mut config = BTreeMap::new();
        config.insert("request_id".to_owned(), request_id.to_string());
        args.insert("request_id".to_owned(), request_id.to_string());
        pipe_message_to_plugin(
            MessageToPlugin::new("filepicker")
                .with_plugin_url("filepicker")
                .with_plugin_config(config)
                .new_plugin_instance_should_have_pane_title(
                    "Select a a folder in which to create the new plugin...",
                )
                .with_args(args),
        );
    }
    pub fn handle_filepicker_response(&mut self, pipe_message: PipeMessage) -> Option<PathBuf> {
        match (pipe_message.payload, pipe_message.args.get("request_id")) {
            (Some(payload), Some(request_id)) => {
                match self
                    .request_ids.iter().position(|p| p == request_id)
                {
                    Some(request_id_position) => {
                        self.request_ids
                            .remove(request_id_position);
                        let chosen_plugin_location = std::path::PathBuf::from(payload);
                        return Some(chosen_plugin_location);
                    },
                    None => {
                        eprintln!("request id not found");
                    },
                }
            },
            _ => {},
        }
        None
    }
}
