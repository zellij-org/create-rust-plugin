![img-2024-11-14-171040](https://github.com/user-attachments/assets/ef9b9d79-369a-4303-afb5-ddbce66bd9d4)


From Zero to "Hello world!" inside a Zellij Rust plugin with one keyboard shortcut.

All you have to do is think of a name for your new plugin and this wizard will:
1. Clone the [`rust-plugin-example`] template repository into a folder by that name in your project directory
2. Create a development workspace for you as a new tab, with your `$EDITOR` and a helper to compile and run your plugin

The only prerequisites are Zellij `0.41.1` and up, and `rust` (with `cargo`) installed on your machine.

More about Zellij plugins: [Zellij Documentation][docs]

### How to run

To try this plugin out (from inside a Zellij session), you can either:
1. `zellij plugin -- https://github.com/zellij-org/create-rust-plugin/releases/latest/download/create-rust-plugin.wasm`
2. Start the `plugin manager` with `Ctrl o` + `p`, press `Ctrl a` to load a new plugin and paste this URL: `https://github.com/zellij-org/create-rust-plugin/releases/latest/download/create-rust-plugin.wasm`

To bind it to a keybinding, add the following to your [keybindings](https://zellij.dev/documentation/keybindings.html):
```kdl
shared {
    bind "Ctrl y" {
        LaunchOrFocusPlugin "https://github.com/zellij-org/create-rust-plugin/releases/latest/download/create-rust-plugin.wasm" {
            project_dir "/home/aram/code" // change-me!
            floating true
            move_to_focused_tab true
        }
    }
}
```

### Configuration
By default, this plugin will assume the folder where it was started is your project folder and clone plugins under it.
For example, if it was started in `/home/aram/code` and you choose `my-cool-plugin` as the plugin name, this wizard will create a new folder for your plugin as `/home/aram/code/my-cool-plugin`.

To change this, run the plugin with the following configuration:
```kdl
project_dir "/path/to/project_dir"
```

[zellij]: https://github.com/zellij-org/zellij
[docs]: https://zellij.dev/documentation/plugins.html
[rust-plugin-example]: https://github.com/zellij-org/rust-plugin-example
