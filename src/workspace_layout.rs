use std::path::PathBuf;

pub fn workspace_layout(plugin_folder: &PathBuf, develop_rust_plugin_ur: &str) -> String {
    format!(
        r#"
        layout {{
            cwd "{}"
            pane size=1 borderless=true {{
                plugin location="tab-bar"
            }}
            pane edit="src/main.rs" size="70%"
            pane size="30%"
            pane size=1 borderless=true {{
                plugin location="status-bar"
            }}
            floating_panes {{
              pane {{
                  plugin location="{develop_rust_plugin_ur}"
              }}
            }}

            swap_floating_layout name="enlarged" {{
                floating_panes max_panes=10 {{
                    pane {{ x "5%"; y 1; width "90%"; height "90%"; }}
                    pane {{ x "5%"; y 2; width "90%"; height "90%"; }}
                    pane {{ x "5%"; y 3; width "90%"; height "90%"; }}
                    pane {{ x "5%"; y 4; width "90%"; height "90%"; }}
                    pane {{ x "5%"; y 5; width "90%"; height "90%"; }}
                    pane {{ x "5%"; y 6; width "90%"; height "90%"; }}
                    pane {{ x "5%"; y 7; width "90%"; height "90%"; }}
                    pane {{ x "5%"; y 8; width "90%"; height "90%"; }}
                    pane {{ x "5%"; y 9; width "90%"; height "90%"; }}
                    pane focus=true {{ x 10; y 10; width "90%"; height "90%"; }}
                }}
            }}

            swap_floating_layout name="spread" {{
                floating_panes max_panes=1 {{
                    pane {{y "50%"; x "50%"; }}
                }}
                floating_panes max_panes=2 {{
                    pane {{ x "1%"; y "25%"; width "45%"; }}
                    pane {{ x "50%"; y "25%"; width "45%"; }}
                }}
                floating_panes max_panes=3 {{
                    pane focus=true {{ y "55%"; width "45%"; height "45%"; }}
                    pane {{ x "1%"; y "1%"; width "45%"; }}
                    pane {{ x "50%"; y "1%"; width "45%"; }}
                }}
                floating_panes max_panes=4 {{
                    pane {{ x "1%"; y "55%"; width "45%"; height "45%"; }}
                    pane focus=true {{ x "50%"; y "55%"; width "45%"; height "45%"; }}
                    pane {{ x "1%"; y "1%"; width "45%"; height "45%"; }}
                    pane {{ x "50%"; y "1%"; width "45%"; height "45%"; }}
                }}
            }}



        }}
    "#,
        plugin_folder.display()
    )
}
