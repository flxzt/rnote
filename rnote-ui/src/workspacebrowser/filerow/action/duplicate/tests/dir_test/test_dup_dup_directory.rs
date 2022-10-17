use super::*;

const ROOT_NAME: &str = "test_dup_dup_directory";
const FILE1: &str = "test_file1.txt";
const SUB_DIR1: &str = "test_sub_dir";
const SUB_DIR1_FILE1: &str = "test_file2.txt";


/// simulates the user who duplicates a directory first
/// and then it duplicates the duplicate.
///
/// # Example
/// 1. `dirA`
/// 2. `dirA` gets duplicated => `dirA`, `dirA.dup`
/// 3. `dirA.dup` gets duplicated => `dirA`, `dirA.dup`, `dirA.dup1`
#[test]
fn test_dup_dup_directory() {
    let test_dir = TestDirectory::new(ROOT_NAME, FILE1, SUB_DIR1, SUB_DIR1_FILE1);

    let first_duplicate_dir = first_duplicate(&test_dir);
    let second_duplicate_dir = second_duplicate(&first_duplicate_dir);

    first_duplicate_dir.assert_existence();
    second_duplicate_dir.assert_existence();

    // cleanup
    test_dir.cleanup();
    first_duplicate_dir.cleanup();
    second_duplicate_dir.cleanup();
}

fn second_duplicate(dir: &TestDirectory) -> TestDirectory {
    let dummy_progress = |_| TransitProcessResult::ContinueOrAbort;
    duplicate_dir(dir.root.clone(), dummy_progress);

    TestDirectory {
        root: PathBuf::from(format!("{}.dup1", ROOT_NAME)),
        .. dir.clone()
    }
}
