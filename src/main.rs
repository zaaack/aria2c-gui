#![feature(plugin, custom_derive,toowned_clone_into)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate open;
extern crate serde_json;
extern crate base64;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use rocket::response::NamedFile;
use rocket_contrib::Template;
use rocket::config::{Config, Environment};
use rocket::State;

mod api;
mod notifier;
mod tray;
mod aria2c;

use tray::{Action, Item, Menu};

#[cfg(test)]
mod tests;

#[derive(Serialize)]
struct Aria2Config {}


#[get("/")]
fn index(config: State<Aria2Config>) -> Template {
    Template::render(
        "index",
        json!({"config": serde_json::to_string_pretty(config.inner()).unwrap()}),
    )
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/dist").join(file)).ok()
}

fn get_port(prefer: u16) -> u16 {
    if let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{}", prefer)) {
        return listener.local_addr().unwrap().port();
    }
    if let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{}", 0)) {
        return listener.local_addr().unwrap().port();
    }
    panic!("Cannot get port!")
}

fn create_menu() -> Menu {
    let icon: &[u8] = include_bytes!("./icon.png");
    Menu {
        icon: base64::encode(icon),
        title: "".to_owned(),
        tooltip: "Aria2cGUI".to_owned(),
        items: vec![
                Item {
                    title: "打开".to_owned(),
                    tooltip: "打开".to_owned(),
                    checked: false,
                    enabled: true,
                },
                Item {
                    title: "退出".to_owned(),
                    tooltip: "退出".to_owned(),
                    checked: false,
                    enabled: true,
                },
            ],
    }
}

fn open_url(port: u16, rpc_port: u16) {
    open::that(format!("http://127.0.0.1:{port}/#!/settings/rpc/set/http/127.0.0.1/{rpc_port}/jsonrpc", port=port, rpc_port=rpc_port)).map_err(|err| {
            println!("open failed: {:?}", err);
        }).unwrap();
}

fn main() {

    let port = get_port(23156);
    let rpc_port = get_port(6800);

    let menu = create_menu();
    let aria2c_tx = aria2c::start_aria2c(rpc_port);
    tray::start_tray(menu, move |action: Action| match action {
        Action::Clicked { item, seq_id } => {
            println!("item:{:?}, seq_id:{:?}", item, seq_id);
            if seq_id == 0 {
                open_url(port, rpc_port);
                Action::None
            } else if seq_id == 1 {
                aria2c_tx.send(-1).unwrap();
                Action::Quit
            } else {
                Action::None
            }
        }
        _ => Action::None,
    });
    let config = Config::build(Environment::Staging)
        .address("127.0.0.1")
        .port(port)
        .extra("template_dir", "static/dist")
        .finalize()
        .unwrap();

    open_url(port, rpc_port);

    rocket::custom(config, true)
        .mount(
            "/",
            routes![
                index,
                api::open_file,
                api::remove_file,
                files,
            ],
        )
        .manage(Aria2Config {})
        .attach(Template::fairing())
        .launch();
}
