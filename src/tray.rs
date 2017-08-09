use std::io::prelude::*;
use std::process::{Command, Stdio};
use std::io::BufReader;
use serde_json;
use std::thread;
use std::time::Duration;

use utils;
// type Item struct {
// 	Title   string `json:"title"`
// 	Tooltip string `json:"tooltip"`
// 	Enabled bool   `json:"enabled"`
// 	Checked bool   `json:"checked"`
// }
// type Menu struct {
// 	Icon    string `json:"icon"`
// 	Title   string `json:"title"`
// 	Tooltip string `json:"tooltip"`
// 	Items   []Item `json:"items"`
// }
//
// type Action struct {
// 	Type  string `json:"type"`
// 	Item  Item   `json:"item"`
// 	Menu  Menu   `json:"menu"`
// 	SeqId int    `json:"seq_id"`
// }
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Item {
    pub title: String,
    pub tooltip: String,
    pub enabled: bool,
    pub checked: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Menu {
    pub icon: String,
    pub title: String,
    pub tooltip: String,
    pub items: Vec<Item>,
}


#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum Action {
    #[serde(rename = "update-item")]
    UpdateItem { item: Item, seq_id: u32 },
    #[serde(rename = "update-menu")]
    UpdateMenu { menu: Menu, seq_id: u32 },
    #[serde(rename = "update-item-and-menu")]
    UpdateItemAndMenu { item: Item, menu: Menu, seq_id: u32 },
    #[serde(rename = "clicked")]
    Clicked { item: Item, seq_id: u32 },
    #[serde(rename = "none")]
    None,
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "quit")]
    Quit,
}

// pub struct Tray {
//     stdin: ChildStdin,
//     stdout: Arc<RefCell<BufReader<ChildStdout>>>,
// }

pub fn start_tray<C>(menu: Menu, callback: C)
where
    C: Fn(Action) -> Action + 'static + Send,
{

    #[cfg(target_os="macos")]
    let tray_bin = include_bytes!("../tray/tray_darwin_release");
    #[cfg(target_os="linux")]
    let tray_bin = include_bytes!("../tray/tray_linux_release");
    let path = utils::init_files("go_tray", tray_bin);
    thread::spawn(move || {
        let mut child = match Command::new(path.as_os_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn() {
            Err(why) => panic!("couldn't spawn tray: {:?}", why),
            Ok(child) => child,
        };
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = BufReader::new(child.stdout.take().unwrap());

        let mut line = String::new();
        stdout.read_line(&mut line).unwrap();
        println!("read: {:?}", line);
        let action = serde_json::from_str::<Action>(line.as_str()).unwrap();

        if Action::Ready != action {
            panic!("Start go tray bin failed");
        }

        let init_data = serde_json::to_string(&menu).unwrap();
        let init_data = init_data + "\n";
        println!("init:{:?}", init_data);
        stdin.write((init_data).as_bytes()).unwrap();

        loop {
            let mut line = String::new();
            stdout.read_line(&mut line).unwrap();
            println!("read: {:?}", line);
            let action = serde_json::from_str::<Action>(line.as_str()).unwrap();
            let action = callback(action);
            if action == Action::Quit {
                child.kill().unwrap();
                // waiting for some cleanning
                thread::sleep(Duration::from_millis(500));
                ::std::process::exit(0);
            }
            let action_str = serde_json::to_string(&action).unwrap() + "\n";
            println!("write:{:?}", action_str);
            stdin.write((action_str).as_bytes()).unwrap();
        }
    });
}
