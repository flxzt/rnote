use super::*;

mod dir_test;
mod test_dup_file;

#[test]
fn test_remove_dup_suffix() {
    // test on filename without ".dup" in name
    let normal = PathBuf::from("normal_file.txt");
    let normal_expected = normal.clone();
    assert_eq!(normal_expected, remove_dup_word(&normal));

    // test with ".dup" name
    let normal_dup = PathBuf::from("normal_file.dup.txt");
    let normal_dup_expected = PathBuf::from("normal_file.txt");
    assert_eq!(normal_dup_expected, remove_dup_word(&normal_dup));

    // test with ".dup1" which means, that a duplicated file has been duplicated
    let normal_dup1 = PathBuf::from("normal_file.dup1.txt");
    let normal_dup1_expected = PathBuf::from("normal_file.txt");
    assert_eq!(normal_dup1_expected, remove_dup_word(&normal_dup1));
}
