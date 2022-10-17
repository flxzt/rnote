use crate::workspacebrowser::filerow::action::duplicate::duplicate_dir;
use fs_extra::dir::TransitProcessResult;
use std::path::PathBuf;

mod test_dup_directory_twice;
mod test_dup_dup_directory;

/// Represents the following directory structure for testing:
/// test_duplicate_directory_root/
///   |- test_file.txt
///   |- test_sub_dir/
///       |- test_file2.txt
#[derive(Debug, Clone)]
pub struct TestDirectory {
    /// '/'
    pub root: PathBuf,
    /// '/<subdir1>'
    pub sub_dir1: PathBuf,
    /// '/<file1>'
    pub file1: PathBuf,
    /// '/<subdir1>/<file1>'
    pub sub_dir1_file1: PathBuf,
}

impl TestDirectory {
    pub fn new(root: &str, file1: &str, sub_dir1: &str, sub_dir1_file1: &str) -> Self {
        let root = PathBuf::from(root);
        let file1 = root.clone().join(file1);
        let sub_dir1 = root.clone().join(sub_dir1);
        let sub_dir1_file1 = sub_dir1.clone().join(sub_dir1_file1);

        let test_dir = Self {
            root,
            file1,
            sub_dir1,
            sub_dir1_file1,
        };

        test_dir.create_entries();
        test_dir
    }

    pub fn assert_existence(&self) {
        assert!(self.root.exists());
        assert!(self.sub_dir1.exists());
        assert!(self.file1.exists());
        assert!(self.sub_dir1_file1.exists());
    }

    fn create_entries(&self) {
        create_dir(&self.root);
        create_dir(&self.sub_dir1);
        create_file(&self.file1);
        create_file(&self.sub_dir1_file1);
    }

    pub fn cleanup(&self) {
        std::fs::remove_dir_all(&self.root).unwrap();
    }
}

/// returns the expected test directory after the first duplication
pub fn first_duplicate(dir: &TestDirectory) -> TestDirectory {
    let dummy_progress = |_| TransitProcessResult::ContinueOrAbort;
    duplicate_dir(dir.root.clone(), dummy_progress);

    TestDirectory {
        root: PathBuf::from(format!("{}.dup", dir.root.display())),
        ..dir.clone()
    }
}

fn create_dir(path: &PathBuf) {
    if let Err(err) = std::fs::create_dir(path) {
        self.cleanup();
        panic!("{}", err);
    }
}

fn create_file(path: &PathBuf) {
    if let Err(err) = std::fs::File::create(path) {
        self.cleanup();
        panic!("{}", err);
    }
}
