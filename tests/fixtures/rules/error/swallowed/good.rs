use std::fs;

pub fn cleanup(path: &str) -> Option<()> {
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(err) => eprintln!("cleanup failed: {err}"),
    }
    let _kept = fs::read_to_string(path).ok();
    fs::remove_file(path).ok()
}
