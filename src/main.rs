use gtk::Application;
use gtk::{glib, prelude::*};
use std::rc::Rc;
use std::{
    collections::VecDeque,
    env,
    fs::{self, File},
    io::{self, BufRead, BufReader, Lines, Result},
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq)]
struct HotkeyCommand {
    hotkey: String,
    command: String,
}

fn main() {
    let app = Application::new(Some("sxhkd-visualiser.zahid.rocks"), Default::default());
    app.connect_activate(|app| {
        if let Ok(config_file_reader) = read_sxhkd_config_file(None) {
            let cleaned_config_content = clean_sxhkd_config_file(config_file_reader);
            let parsed_config_content = parse_sxhkd_config_file_content(cleaned_config_content);
            build_ui(app, parsed_config_content);
        }
    });
    app.run();
}

fn build_ui(application: &gtk::Application, hotkey_data: Vec<HotkeyCommand>) {
    let window = gtk::ApplicationWindow::new(application);
    window.set_title("SXHKD Keybindings");
    window.set_border_width(10);
    window.set_default_size(400, 850);
    window.set_position(gtk::WindowPosition::Center);
    window.set_type_hint(gtk::gdk::WindowTypeHint::Dialog);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);
    window.add(&vbox);

    let label = gtk::Label::new(Some("KEYBINDINGS"));
    vbox.add(&label);

    let sw = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    sw.set_shadow_type(gtk::ShadowType::EtchedIn);
    sw.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    vbox.add(&sw);

    let model = Rc::new(create_model(hotkey_data));
    let treeview = gtk::TreeView::with_model(&*model);
    treeview.set_vexpand(true);

    sw.add(&treeview);
    add_columns(&treeview);
    window.show_all();
}

fn create_model(hotkey_data: Vec<HotkeyCommand>) -> gtk::ListStore {
    let col_types: [glib::Type; 2] = [glib::Type::STRING, glib::Type::STRING];
    let store = gtk::ListStore::new(&col_types);

    for d in hotkey_data.iter() {
        let columns_and_values: [(u32, &dyn ToValue); 2] = [(0, &d.hotkey), (1, &d.command)];
        store.set(&store.append(), &columns_and_values);
    }

    return store;
}

fn add_columns(treeview: &gtk::TreeView) {
    // Column for Hotkeys
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Hotkey");
        column.add_attribute(&renderer, "text", 0);
        treeview.append_column(&column);
    }

    // Column for commands
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Command");
        column.add_attribute(&renderer, "text", 1);
        treeview.append_column(&column);
    }
}

fn read_sxhkd_config_file(config_file_location: Option<String>) -> Result<Lines<BufReader<File>>> {
    let config_file_path: PathBuf;
    if config_file_location.is_some() {
        config_file_path = Path::new(config_file_location.unwrap().as_str()).to_path_buf();
    } else {
        let home_dir: String = env::var_os("HOME")
            .expect("Can not get HOME env variable")
            .into_string()
            .unwrap();
        let sxhkd_config_file_location = ".config/sxhkd/sxhkdrc";
        config_file_path = Path::new(home_dir.as_str()).join(sxhkd_config_file_location);
    }
    let file = fs::File::open(config_file_path)?;
    return Ok(io::BufReader::new(file).lines());
}

fn clean_sxhkd_config_file(config_file_content: Lines<BufReader<File>>) -> VecDeque<String> {
    let mut file_content: VecDeque<String> = VecDeque::new();

    for line_result in config_file_content.into_iter() {
        let line = line_result.unwrap();
        if line.trim().is_empty() || line.trim().chars().next() == Some('#') {
            continue;
        }
        file_content.push_back(line.trim().to_string())
    }

    return file_content;
}

fn parse_sxhkd_config_file_content(mut clean_file_content: VecDeque<String>) -> Vec<HotkeyCommand> {
    let mut hotkey_commands = Vec::new();

    while !clean_file_content.is_empty() {
        let definition = clean_file_content.pop_front().unwrap();
        let mut command_str = String::new();
        loop {
            let partial_command = clean_file_content.pop_front().unwrap();
            if partial_command.chars().last() == Some('\\') {
                command_str.push_str(partial_command.trim_end_matches('\\'));
                command_str.push_str(" ")
            } else {
                command_str.push_str(partial_command.as_str());
                break;
            }
        }
        hotkey_commands.push(HotkeyCommand {
            hotkey: definition,
            command: command_str,
        })
    }

    return hotkey_commands;
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        fs::{remove_file, File},
        io::Write,
    };

    use crate::{
        clean_sxhkd_config_file, parse_sxhkd_config_file_content, read_sxhkd_config_file,
        HotkeyCommand,
    };

    #[test]
    fn test_read_sxhkd_config_file() {
        let content = b"hotkey\ncommand\n";
        let file_name = "test_sxhkdrc";
        let mut temp_file = File::create(file_name).unwrap();
        temp_file.write_all(content).unwrap();
        temp_file.flush().unwrap();

        let result = read_sxhkd_config_file(Some(file_name.to_string())).unwrap();
        let result_vec: Vec<String> = result.into_iter().map(|x| x.unwrap()).collect();
        assert_eq!(result_vec, vec!["hotkey", "command"]);

        remove_file(file_name).unwrap();
    }

    #[test]
    fn test_clean_sxhkd_config_file() {
        let content = b"\n\nhotkey\n#comment\ncommand\n   \n";
        let file_name = "test_sxhkdrc2";
        let mut temp_file = File::create(file_name).unwrap();
        temp_file.write_all(content).unwrap();
        temp_file.flush().unwrap();

        let file_lines = read_sxhkd_config_file(Some(file_name.to_string())).unwrap();
        let cleaned_content = clean_sxhkd_config_file(file_lines);

        assert_eq!(cleaned_content, vec!["hotkey", "command"]);

        remove_file(file_name).unwrap();
    }

    #[test]
    fn test_basic_parse_sxhkd_config_file() {
        let content: VecDeque<String> = VecDeque::from(["key".to_string(), "value".to_string()]);
        let result = parse_sxhkd_config_file_content(content);
        let expected = vec![HotkeyCommand {
            hotkey: "key".to_string(),
            command: "value".to_string(),
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_multiple_command_line_parse_sxhkd_config_file() {
        let content: VecDeque<String> = VecDeque::from([
            "key".to_string(),
            "value\\".to_string(),
            "value_next_line".to_string(),
        ]);
        let result = parse_sxhkd_config_file_content(content);
        let expected = vec![HotkeyCommand {
            hotkey: "key".to_string(),
            command: "value value_next_line".to_string(),
        }];
        assert_eq!(result, expected);
    }
}
