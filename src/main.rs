use std::env;
use std::path::Path;
use pelite::pe64::{PeFile, Pe};
use regex::Regex;
use walkdir::WalkDir;
use std::fmt;
use std::error::Error;

// Define a custom error type to handle both 'std::io::Error' and 'pelite::Error'
#[derive(Debug)]
enum ExportError {
    IoError(std::io::Error),
    PeError(pelite::Error),
    PeFindError(pelite::resources::FindError),
}

impl fmt::Display for ExportError {  // fmt is in scope for Display trait
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ExportError::IoError(ref err) => write!(f, "IO error: {}", err),
            ExportError::PeError(ref err) => write!(f, "PE error: {}", err),
            ExportError::PeFindError(ref err) => write!(f, "PE find error: {}", err),
        }
    }
}

impl Error for ExportError {}  // Error trait is now in scope

impl From<std::io::Error> for ExportError {
    fn from(err: std::io::Error) -> ExportError {
        ExportError::IoError(err)
    }
}

impl From<pelite::Error> for ExportError {
    fn from(err: pelite::Error) -> ExportError {
        ExportError::PeError(err)
    }
}

impl From<pelite::resources::FindError> for ExportError {
    fn from(err: pelite::resources::FindError) -> ExportError {
        ExportError::PeFindError(err)
    }
}

fn get_manifest(file_path: &str) -> Result<String, ExportError> {
    let buffer = std::fs::read(file_path)?;           // Converts std::io::Error to ExportError
    let pe = PeFile::from_bytes(&buffer)?;  // Converts pelite::Error to ExportError

     // Get resources directory
    let resources = pe.resources()?;

    return resources.manifest().map(|s| s.to_string()).map_err(ExportError::from);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        let program = Path::new(&args[0])
            .file_name()
            .unwrap()
            .to_string_lossy();
        eprintln!("Usage: {} <root folder to check>", program);
        std::process::exit(1);
    }
    
    let path = &args[1];

    if !Path::new(path).is_dir() {
        eprintln!("{} is not a valid folder", path);
        std::process::exit(1);
    }

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        // Unwrap each entry
        let path = entry.path();

        // Check that it is a file and ends with ".exe" (case-insensitive)
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.eq_ignore_ascii_case("exe") {
                    let manifest = get_manifest(path.to_str().unwrap());
                    match manifest {
                        Ok(m) => {
                            // Pattern 1: autoElevate true
                            let re_auto = Regex::new(r"<\s*autoElevate\s*>\s*true\s*<\s*/\s*autoElevate\s*>").unwrap();

                            // Pattern 2: requestedExecutionLevel requireAdministrator
                            let re_exec = Regex::new(
                                r#"<\s*requestedExecutionLevel[^>]*level\s*=\s*"(requireAdministrator)""#
                            ).unwrap();

                            // Check both
                            if re_auto.is_match(&m) && re_exec.is_match(&m) {
                                println!("Found EXE: {}", path.display());
                            }
                        },
                        Err(_e) => { }
                    }
                }
                
            }
        }
    }
}
