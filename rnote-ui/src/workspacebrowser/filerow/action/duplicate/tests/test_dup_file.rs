use std::{path::PathBuf, fs::File};

use crate::workspacebrowser::filerow::action::duplicate::get_destination_path;

/// simulates the user who duplicates the same file twice
#[test]
fn test_get_destination_path_file() {
    let filename = PathBuf::from("test_get_destination_path.txt");
    File::create(&filename).unwrap();

    let dup0_filename = get_destination_path(&filename).unwrap();
    if let Err(err) = File::create(&dup0_filename) {
        // "emergence" cleanup
        std::fs::remove_file(filename).unwrap();
        panic!("{}", err);
    }

    let dup1_filename = get_destination_path(&filename).unwrap();

    let expected_dup0 = PathBuf::from("test_get_destination_path.dup.txt");
    let expected_dup1 = PathBuf::from("test_get_destination_path.dup1.txt");

    assert_eq!(dup0_filename, expected_dup0);
    assert_eq!(dup1_filename, expected_dup1);

    // cleanup
    std::fs::remove_file(filename).unwrap();
    std::fs::remove_file(dup0_filename).unwrap();
}
