#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::io;
use rocket::response::NamedFile;
use rocket::config::{Config, Environment};


#[cfg(test)]
mod tests;


#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open("static/dist/index.html")
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

fn main() {
    let port = get_port(23156);
    let config = Config::build(Environment::Staging)
        .address("127.0.0.1")
        .port(port)
        .finalize()
        .unwrap();

    rocket::custom(config, true)
        .mount("/", routes![index, files])
        .launch();
}
