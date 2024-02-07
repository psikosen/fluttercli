use std::fs;

fn is_flutter_project_dir() -> bool {
    fs::metadata("pubspec.yaml").is_ok() && fs::metadata("lib/").is_ok()
}

fn main() {
    if is_flutter_project_dir() {
        println!("You are in a Flutter project directory.");
    } else {
        println!("You are NOT in a Flutter project directory.");
    }
}
