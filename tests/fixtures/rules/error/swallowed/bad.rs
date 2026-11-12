use std::fs;

pub fn cleanup(path: &str) {
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(_) => {} // expect: error/swallowed
    }
    match fs::remove_dir(path) {
        Ok(()) => (),
        Err(_) => (), // expect: error/swallowed
    }
    fs::remove_file(path).ok(); // expect: error/swallowed
}
