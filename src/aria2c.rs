extern crate futures;
extern crate hyper;
extern crate tokio_core;

use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};
use serde_json;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::{thread, time};
use std::io;
use std::fs;
use std::env;
use self::hyper::Client;
use self::futures::{Future, Stream};
use self::tokio_core::reactor::Core;
use self::hyper::{Method, Request};
use self::hyper::header::{ContentLength, ContentType};

use notifier;

#[derive(Serialize, Deserialize, Debug)]
struct TaskFile {
    path: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Task {
    gid: String,
    files: Vec<TaskFile>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JSONRPCRes {
    result: Vec<Task>,
}

pub fn start_aria2c(port: u16) -> Sender<i8> {
    let (tx, rx): (Sender<i8>, Receiver<i8>) = channel();
    thread::spawn(move || {
        let mut download_dir = env::home_dir().unwrap();
        download_dir.push("Downloads");
        let mut tmp = PathBuf::new();
        download_dir.clone_into(&mut tmp);
        fs::create_dir_all(download_dir).err().map(|err| {
            println!("{:?}", err);
        });
        let mut child = match Command::new("aria2c")
            .arg("--enable-rpc")
            .arg(format!("--rpc-listen-port={port}", port = port))
            .arg("--rpc-allow-origin-all")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .current_dir(tmp)
            .spawn() {
            Err(why) => panic!("couldn't spawn aria2c: {:?}", why),
            Ok(child) => child,
        };
        loop {
            match rx.recv() {
                Ok(-1) => {
                    child.kill().unwrap();
                    println!("Exit aria2c");
                    break;
                }
                _ => (),
            }
        }

    });


    thread::spawn(move || {
        let mut task_len = 0;
        loop {
            thread::sleep(time::Duration::from_secs(7));
            let mut core = Core::new().unwrap();
            let handle = core.handle();
            let client = Client::new(&handle);
            let json = r#"{
                "jsonrpc":"2.0",
                "method":"aria2.tellStopped",
                "id":"aria2c-gui-rs",
                "params":[
                    -1,
                    1000,
                    [
                        "gid",
                        "files"
                    ]
                ]
            }"#;
            let uri = format!("http://127.0.0.1:{port}/jsonrpc", port = port)
                .parse()
                .unwrap();
            let mut req = Request::new(Method::Post, uri);
            req.headers_mut().set(ContentType::json());
            req.headers_mut().set(ContentLength(json.len() as u64));
            req.set_body(json);
            let post = client.request(req).and_then(|res| {

                res.body().concat2().and_then(|body| {
                    let v: JSONRPCRes = serde_json::from_slice(&body).map_err(|e| {
                        io::Error::new(io::ErrorKind::Other, e)
                    })?;
                    println!("value:{:?} {}", v, task_len);
                    let len = v.result.len();
                    if len > task_len {
                        v.result
                            .iter()
                            .take(len - task_len)
                            .map(|task| if task.files.len() > 0 {
                                let file = &task.files[0];
                                let path = Path::new(&file.path);
                                let filename = path.file_name().unwrap();
                                notifier::notify(
                                    &format!("下载完成: {}", filename.to_string_lossy()),
                                    &file.path,
                                );
                                task
                            } else {
                                task
                            })
                            .collect::<Vec<&Task>>();
                    }
                    task_len = len;
                    Ok(())
                })
            });
            core.run(post).unwrap();
        }
    });
    return tx;
}
