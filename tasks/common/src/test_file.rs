use std::{fmt, fs::read_to_string, str::FromStr};

use crate::project_root;
use crate::request::agent;

pub struct TestFiles {
    files: Vec<TestFile>,
}

impl Default for TestFiles {
    fn default() -> Self {
        Self::new()
    }
}

impl TestFiles {
    pub fn new() -> Self {
        let files = Self::get_files().into_iter().map(|file| TestFile::new(&file)).collect();
        Self { files }
    }

    pub fn minimal() -> Self {
        let files = Self::get_files()
            .into_iter()
            .filter(|name| ["react", "antd", "typescript"].iter().any(|f| name.contains(f)))
            .map(|file| TestFile::new(&file))
            .collect();
        Self { files }
    }

    pub fn files(&self) -> &Vec<TestFile> {
        &self.files
    }

    fn get_files() -> Vec<String> {
        let root = project_root();
        read_to_string(root.join("./tasks/libs.txt"))
            .unwrap()
            .lines()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    }
}

pub struct TestFile {
    pub url: String,
    pub file_name: String,
    pub source_text: String,
}

impl TestFile {
    /// # Errors
    /// # Panics
    pub fn new(url: &str) -> Self {
        let (file_name, source_text) = Self::get_source_text(url).unwrap();
        Self { url: url.to_string(), file_name, source_text }
    }

    /// # Errors
    /// # Panics
    pub fn get_source_text(lib: &str) -> Result<(String, String), String> {
        let url = url::Url::from_str(lib).map_err(err_to_string)?;

        let segments = url.path_segments().ok_or_else(|| "lib url has no segments".to_string())?;

        let filename = segments.last().ok_or_else(|| "lib url has no segments".to_string())?;

        let file = project_root().join("target").join(filename);

        if let Ok(code) = std::fs::read_to_string(&file) {
            println!("[{filename}] - using [{}]", file.display());
            Ok((filename.to_string(), code))
        } else {
            println!("[{filename}] - Downloading [{lib}] to [{}]", file.display());
            match agent().get(lib).call() {
                Ok(response) => {
                    let mut reader = response.into_reader();

                    let _drop = std::fs::remove_file(&file);
                    let mut writer = std::fs::File::create(&file).map_err(err_to_string)?;
                    let _drop = std::io::copy(&mut reader, &mut writer);

                    std::fs::read_to_string(&file)
                        .map_err(err_to_string)
                        .map(|code| (filename.to_string(), code))
                }
                Err(e) => Err(format!("{e:?}")),
            }
        }
    }
}

fn err_to_string<E: fmt::Debug>(e: E) -> String {
    format!("{e:?}")
}
