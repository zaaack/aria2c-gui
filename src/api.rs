extern crate open;
use rocket_contrib::{Json, Value};
use std::process::Command;

#[derive(FromForm)]
pub struct FileForm {
    pub file: String,
    pub cmd: Option<String>,
}

#[get("/api/open?<query>")]
pub fn open_file(query: FileForm) -> Json<Value> {
    if let Err(result) = open::that(query.file) {
        return Json(
            json!({"status": "failed", "error": format!("{:?}", result)}),
        );
    }
    Json(json!({ "status": "ok" }))
}


#[get("/api/remove?<query>")]
pub fn remove_file(query: FileForm) -> Json<Value> {
    let cmd = query.cmd.unwrap_or("rm".to_owned());
    let output = Command::new(cmd.as_str())
        .arg(query.file.to_string())
        .output()
        .expect("failed to execute process");
    if output.status.success() {
        return Json(json!({ "status": "ok" }));
    }
    Json(
        json!({"status": "failed", "error": format!("{:?}", format!("stderr: {}", String::from_utf8_lossy(&output.stderr)))}),
    )
}
