use std::fs::File;

use super::*;

mod test_dup_directory;
mod test_dup_file;

#[test]
fn test_remove_dup_suffix() {
    // test on filename without ".dup" in name
    let normal = OsString::from("normal_file.txt");
    let normal_expected = normal.clone();
    assert_eq!(normal_expected, remove_dup_suffix(&normal));

    // test with ".dup" name
    let normal_dup = OsString::from("normal_file.dup.txt");
    let normal_dup_expected = OsString::from("normal_file.txt");
    assert_eq!(normal_dup_expected, remove_dup_suffix(&normal_dup));

    // test with ".dup1" which means, that a duplicated file has been duplicated
    let normal_dup1 = OsString::from("normal_file.dup1.txt");
    let normal_dup1_expected = OsString::from("normal_file.txt");
    assert_eq!(normal_dup1_expected, remove_dup_suffix(&normal_dup1));
}
