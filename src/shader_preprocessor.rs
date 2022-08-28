/*
use std::fs;
use std::io;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum PreprocessorError {
    FileNotFound(String),
    FolderNotFound(String),
    IOError(io::Error),
}

impl Display for PreprocessorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::FileNotFound(s) => {
                write!(f, "file not found \"{}\"", s);
            }
            Self::FolderNotFound(f) => {
                write!(f, "folder not found \"{}\"", f);
            }
            Self::IOError(e) => {
                write!(f, "ioerror: {}", e);
            }
        }
    }
}

pub struct PreprocessorResult {
    source: String,
    files_read: Vec<String>,
}


pub fn process_shader(source_fn: &str, load_folders: &[&str]) -> Result<PreprocessorResult, PreprocessorError> {
    // Load source/entry-point


}
*/
