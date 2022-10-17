use super::*;

const ROOT_NAME: &str = "test_dup_directory_twice";
const FILE1: &str = "test_file1.txt";
const SUB_DIR1: &str = "test_sub_dir";
const SUB_DIR1_FILE1: &str = "test_file2.txt";

/// simulates the user who duplicates the *same* directory twice
#[test]
fn test_dup_directory() {
    let test_dir = TestDirectory::new(ROOT_NAME, FILE1, SUB_DIR1, SUB_DIR1_FILE1);

    let first_duplicate = first_duplicate(&test_dir);
    let second_duplicate = second_duplicate(&test_dir);

    first_duplicate.assert_existence();
    second_duplicate.assert_existence();

    // cleanup
    test_dir.cleanup();
    first_duplicate.cleanup();
    second_duplicate.cleanup();
}

/// returns the expected test directory after the second duplication
fn second_duplicate(dir: &TestDirectory) -> TestDirectory {
    let dummy_progress = |_| {TransitProcessResult::ContinueOrAbort};
    duplicate_dir(dir.root.clone(), dummy_progress);

    TestDirectory {
        root: PathBuf::from(format!("{}.dup1", dir.root.display())),
        .. dir.clone()
    }
}
