use image::GenericImageView;
use std::path::Path;

fn main() {
    let folder = "assets/images/test/player";
    let paths = std::fs::read_dir(folder).expect("Failed to read directory");

    println!("Checking dimensions in {}", folder);

    for entry in paths {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("png") {
            match image::open(&path) {
                Ok(img) => {
                    println!(
                        "{}: {}x{}",
                        path.file_name().unwrap().to_string_lossy(),
                        img.width(),
                        img.height()
                    );
                }
                Err(e) => {
                    println!("Failed to open {}: {}", path.display(), e);
                }
            }
        }
    }
}
