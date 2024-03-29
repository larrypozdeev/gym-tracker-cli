use crate::errors::ResultError::OtherError;
use crate::errors::{FileError, Result, ResultError};
use crate::user_profile::{UserProfile, Users};
use std::fs::File;
use std::io::prelude::*;

use std::io::ErrorKind;
use std::path::Path;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum FileContents {
    Users(Users),
    UserProfile(UserProfile),
}

impl PartialEq for FileContents {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (FileContents::Users(users1), FileContents::Users(users2)) => users1 == users2,
            (
                FileContents::UserProfile(user_profile1),
                FileContents::UserProfile(user_profile2),
            ) => user_profile1 == user_profile2,
            _ => false,
        }
    }
}
pub fn read_file(path_str: &str) -> Result<FileContents> {
    let path = Path::new(path_str);

    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(e) => {
            return match e.kind() {
                ErrorKind::NotFound => Ok(FileContents::Users(Users::new())),
                _ => Err(OtherError(e.to_string())),
            }
        }
    };

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| ResultError::FileError(FileError::IoError(e)))?;

    serde_json::from_str(&contents).map_err(|e| ResultError::FileError(FileError::SerdeError(e)))
}

pub fn update_file(path: &str, contents: &FileContents) -> Result<()> {
    let path = Path::new(path);

    // is file valid json?
    let _ = read_file(path.to_str().unwrap())?;

    let contents = serde_json::to_string(&contents).map_err(FileError::SerdeError)?;

    let mut file = File::create(&path).map_err(FileError::IoError)?;

    file.write_all(contents.as_bytes())
        .map_err(|e| ResultError::FileError(FileError::IoError(e)))
}

#[cfg(test)]
pub mod test_utils {
    use crate::user_profile::{UserProfile, Users};
    use crate::utils::{update_file, FileContents};
    use std::fs;
    use std::path::Path;

    pub const TEST_FILE: &str = "test_files/test.json";
    pub const EMPTY_FILE: &str = "test_files/empty.json";

    pub fn clear_path(path: &str) {
        let path = Path::new(path);
        if path.exists() {
            fs::remove_file(path).unwrap();
        }
    }

    #[allow(dead_code)]
    pub fn create_test_user_profile() {
        let user_profile = UserProfile::new("test".to_string());
        // save user profile in test file
        update_file(TEST_FILE, &FileContents::UserProfile(user_profile.clone())).unwrap();

        let mut users = Users::new();
        users.add_user(user_profile);
        update_file(TEST_FILE, &FileContents::Users(users)).unwrap();
    }
}

// tests for utils.rs
#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::*;

    // tests for read_file
    #[test]
    fn test_read_file() {
        let mut users = Users::new();
        users.add_user(UserProfile::new("test".to_string()));
        let unique_path = TEST_FILE.to_string() + "5";
        update_file(&unique_path, &FileContents::Users(users.clone())).unwrap();

        let contents = read_file(&unique_path);
        assert_eq!(contents, Ok(FileContents::Users(users)));
        clear_path(unique_path.as_str());
    }

    #[test]
    fn test_read_file_not_found() {
        let contents = read_file(EMPTY_FILE);
        assert_eq!(contents, Ok(FileContents::Users(Users::new())));
        clear_path(EMPTY_FILE);
    }

    //tests for update_file
    #[test]
    fn test_update_file() {
        let unique_path = TEST_FILE.to_string() + "2";

        // also tests for invalid path, as it it will create the file
        let mut users = Users::new();
        users.add_user(UserProfile::new("test".to_string()));

        update_file(&unique_path, &FileContents::Users(users.clone())).unwrap();
        let contents = read_file(&unique_path).unwrap();

        assert_eq!(contents, FileContents::Users(users));
        clear_path(&unique_path);
    }

    #[test]
    fn test_update_file_invalid_directory() {
        let mut users = Users::new();
        users.add_user(UserProfile::new("test".to_string()));

        let result = update_file(
            "test_files/invalid_path/invalid.json",
            &FileContents::Users(users),
        );
        assert!(matches!(
            result,
            Err(ResultError::FileError(FileError::IoError(_)))
        ));
    }
}
