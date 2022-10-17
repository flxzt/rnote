use std::path::PathBuf;

use fs_extra::dir::TransitProcessResult;

use crate::workspacebrowser::filerow::action::duplicate::duplicate_dir;

const ROOT_NAME: &str = "test_duplicate_directory_root";
const FILE1: &str = "test_file1.txt";
const SUB_DIR1: &str = "test_sub_dir";
const SUB_DIR1_FILE1: &str = "test_file2.txt";

/// The test directory structure
#[derive(Debug, Clone)]
struct TestDirectory {
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
    pub fn new() -> Self {
        let root = PathBuf::from(ROOT_NAME);
        let file1 = root.clone().join(FILE1);
        let sub_dir1 = root.clone().join(SUB_DIR1);
        let sub_dir1_file1 = sub_dir1.clone().join(SUB_DIR1_FILE1);

        Self {
            root,
            file1,
            sub_dir1,
            sub_dir1_file1,

        }
    }

    pub fn assert_existence(&self) {
        assert!(self.root.exists());
        assert!(self.sub_dir1.exists());
        assert!(self.file1.exists());
        assert!(self.sub_dir1_file1.exists());
    }

    pub fn create_entries(&self) {
        self.create_dir(&self.root);
        self.create_dir(&self.sub_dir1);
        self.create_file(&self.file1);
        self.create_file(&self.sub_dir1_file1);
    }

    pub fn cleanup(&self) {
        std::fs::remove_dir_all(&self.root).unwrap();
    }

    fn create_dir(&self, path: &PathBuf) {
        if let Err(err) = std::fs::create_dir(path) {
            self.cleanup();
            panic!("{}", err);
        }
    }

    fn create_file(&self, path: &PathBuf) {
        if let Err(err) = std::fs::File::create(path) {
            self.cleanup();
            panic!("{}", err);
        }
    }
}

/// simulates the user who duplicates a directory twice
#[test]
fn test_dup_directory() {
    let test_dir = TestDirectory::new();
    test_dir.create_entries();

    let first_duplicate = first_duplicate(&test_dir);
    let second_duplicate = second_duplicate(&test_dir);

    first_duplicate.assert_existence();
    second_duplicate.assert_existence();

    // cleanup
    test_dir.cleanup();
    first_duplicate.cleanup();
    second_duplicate.cleanup();
}

fn first_duplicate(dir: &TestDirectory) -> TestDirectory {
    let dummy_progress = |_| {TransitProcessResult::ContinueOrAbort};
    duplicate_dir(dir.root.clone(), dummy_progress);

    TestDirectory {
        root: PathBuf::from(format!("{}.dup", dir.root.display())),
        .. dir.clone()
    }
}

fn second_duplicate(dir: &TestDirectory) -> TestDirectory {
    let dummy_progress = |_| {TransitProcessResult::ContinueOrAbort};
    duplicate_dir(dir.root.clone(), dummy_progress);

    TestDirectory {
        root: PathBuf::from(format!("{}.dup1", dir.root.display())),
        .. dir.clone()
    }
}
