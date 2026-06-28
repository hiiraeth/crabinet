use std::{env, fs, time::Instant};
use std::path::{Path, PathBuf};

// adding an extension or category here wires up both folder creation and sorting
const CATEGORIES: &[(&str, &[&str])] = &[
    ("img", &[
        "png", "jpg", "jpeg", "webp", "gif", "bmp", "tiff", "tif",
        "heic", "heif", "svg", "ico", "raw", "cr2", "nef", "arw", "rw2",
    ]),
    ("video", &[
        "mp4", "mkv", "mov", "avi", "wmv", "flv", "webm",
        "m4v", "mpg", "mpeg", "3gp",
    ]),
    ("audio", &[
        "mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "aiff",
    ]),
    ("txt", &[
        "txt", "md", "rtf", "log", "tex",
    ]),
    ("docs", &[
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
        "odt", "ods", "odp", "csv",
    ]),
    ("apps", &[
        "dmg", "pkg", "app", "exe", "msi", "deb", "rpm", "appimage",
        "jar",
    ]),
    ("compressed", &[
        "zip", "rar", "7z", "tar", "gz", "bz2", "xz", "tgz", "zst",
    ]),
    ("code", &[
        "rs", "py", "js", "ts", "c", "cpp", "h", "java",
        "go", "rb", "sh", "json", "toml", "yaml", "yml",
        "html", "glsl", "ipynb"
    ]),
];

fn main() -> std::io::Result<()> {
    let (input_dir, output_dir, dry_run) = setup();

    // start the clock after validation so we only time real work
    let time_elapsed = Instant::now();

    // skipped in dry-run so a preview never touches the disk
    if !dry_run {
        println!("organizing files...");
        create_output_dirs(&output_dir).expect("failed to create directories");
    }

    organize_files(&input_dir, &output_dir, dry_run)?;

    if !dry_run {
        println!("finished organizing files in {:.2?}", time_elapsed.elapsed());
    }
    
    Ok(())
}

fn setup() -> (PathBuf, PathBuf, bool) {
    let args: Vec<String> = env::args().collect();

    // strip the flag so it doesn't throw off positional indexing below
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let positionals: Vec<&String> = args.iter()
        .skip(1)
        .filter(|a| !a.starts_with('-'))
        .collect();

    // get() instead of indexing so a missing arg can't panic
    let Some(input_path) = positionals.get(0) else {
        eprintln!("Usage: {} <input folder> <output folder>", args[0]);
        std::process::exit(1);
    };

    let Some(output_path) = positionals.get(1) else {
        eprintln!("Usage: {} <input folder> <output folder>", args[0]);
        std::process::exit(1);
    };

    // owned PathBuf so these can outlive `args`
    let input_dir = PathBuf::from(input_path);
    let output_dir = PathBuf::from(output_path);

    let input_dir_exists = check_if_dir_exists(&input_dir);

    if !input_dir_exists {
        eprintln!("input directory does not exist. enter a valid directory");
        std::process::exit(1);
    }

    (input_dir, output_dir, dry_run)
}

fn match_ext_to_dir(extension: &str) -> &str {
    for (name, extensions) in CATEGORIES {
        // lowercase so e.g. "JPG" still matches the table
        if extensions.contains(&extension.to_lowercase().as_str()) {
            return name;
        }
    }

    "misc"
}

fn organize_files(input_dir: &Path, output_dir_parent: &Path, dry_run: bool) -> std::io::Result<()> {
    let mut moved_counter = 0;
    let mut skipped_counter = 0;

    // non-recursive: only immediate contents, no descent into subfolders
    for entry in fs::read_dir(input_dir)? {
        let path = entry?.path();

        if path.is_file() {
            let extension = path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");

            // nothing to categorize a file by if it has no extension
            if extension.is_empty() {
                eprintln!("skipping file without extension: {}", path.display());
                skipped_counter += 1;
                continue;
            }

            let output_dir = match_ext_to_dir(&extension);
            let dest_dir = format!("{}/{}/", output_dir_parent.to_string_lossy(), output_dir);

            let file_name = path.file_name().unwrap().to_str().unwrap();

            let dest_file = dest_dir + file_name;
            let dest_file_path = Path::new(&dest_file);

            let Some(dest_file) = check_collisions(dest_file_path) else {
                skipped_counter += 1;
                continue;
            };

            if dry_run {
                println!("would move {} -> {}", file_name, dest_file.display());
                moved_counter += 1;
            } else {
                // best-effort: log the failure and keep going instead of aborting
                if let Err(e) = fs::rename(path, &dest_file) {
                    eprintln!("failed to move file {}: {}", dest_file.display(), e);
                } else {
                    moved_counter += 1;
                }
            }
        }
    }

    if dry_run {
        println!("\nwould move {} files, skipped {} files", moved_counter, skipped_counter);
    } else {
        println!("\nmoved {} files, skipped {} files", moved_counter, skipped_counter);
    }

    Ok(())
}

// returns a free destination path, numbering it "(n)" if the name is taken
fn check_collisions(dest_file_path: &Path) -> Option<PathBuf> {
    let collision = dest_file_path.try_exists().unwrap_or(false);

    if collision {
        let mut file_counter = 0;

        let Some(dest_file_stem) = dest_file_path
            .file_stem()
            .and_then(|s| s.to_str()) else {
            eprintln!("failed to get file stem for collision on file {}. file not moved",
                      dest_file_path.display());
            return None;
        };

        let Some(dest_file_extension) = dest_file_path
            .extension()
            .and_then(|s| s.to_str()) else {
            eprintln!("failed to get file extension for collision on file {}",
                      dest_file_path.display());
            return None;
        };

        // bump the counter until we find a name that isn't taken
        loop {
            file_counter += 1;
            let filename = format!("{} ({}).{}", dest_file_stem, file_counter, dest_file_extension);
            let candidate = dest_file_path.parent().unwrap().join(&filename);
            if !candidate.exists() {
                return Some(candidate);
            }
        }
    }

    Some(PathBuf::from(dest_file_path))
}

// treats an io error as "doesn't exist" while logging it, rather than propagating
fn check_if_dir_exists(dir: &Path) -> bool {
    match dir.try_exists() {
        Ok(true) => { true },
        Ok(false) => { false },
        Err(_) => { eprintln!("error fetching directory {}", dir.display()); false }
    }
}

// create_dir_all is idempotent, so re-running is safe
fn create_output_dirs(output_dir: &Path) -> Result<(), std::io::Error> {
    // misc isn't in CATEGORIES (it's the fallback), so create it explicitly
    fs::create_dir_all(output_dir.join("misc"))?;

    for directory in CATEGORIES {
        let output = output_dir.join(directory.0);
        let path = Path::new(&output);

        if let Err(e) = fs::create_dir_all(path) {
            eprintln!("error creating output directory {}: {}", output.display(), e);
            std::process::exit(1);
        };
    }

    Ok(())
}