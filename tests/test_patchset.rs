extern crate unidiff;

use unidiff::PatchSet;

#[test]
fn test_parse_sample0_diff() {
    let buf = include_str!("fixtures/sample0.diff");

    let mut patch = PatchSet::new();
    patch.parse(&buf).unwrap();

    // three file in the patch
    assert_eq!(3, patch.len());
    // three hunks
    assert_eq!(3, patch[0].len());

    // first file is modified
    assert!(patch[0].is_modified_file());
    assert!(!patch[0].is_added_file());
    assert!(!patch[0].is_removed_file());

    // Hunk 1: five additions, no deletions, a section header
    assert_eq!(6, patch[0][0].added());
    assert_eq!(0, patch[0][0].removed());
    assert_eq!("Section Header", &patch[0][0].section_header);

    // Hunk 2: 2 additions, 8 deletions, no section header
    assert_eq!(2, patch[0][1].added());
    assert_eq!(8, patch[0][1].removed());
    assert_eq!("", &patch[0][1].section_header);

    // Hunk 3: four additions, no deletions, no section header
    assert_eq!(4, patch[0][2].added());
    assert_eq!(0, patch[0][2].removed());
    assert_eq!("", &patch[0][2].section_header);

    // Check file totals
    assert_eq!(12, patch[0].added());
    assert_eq!(8, patch[0].removed());

    // second file is added
    assert!(!patch[1].is_modified_file());
    assert!(patch[1].is_added_file());
    assert!(!patch[1].is_removed_file());

    // third file is removed
    assert!(!patch[2].is_modified_file());
    assert!(!patch[2].is_added_file());
    assert!(patch[2].is_removed_file());
}

#[test]
fn test_parse_git_diff() {
    let buf = include_str!("fixtures/git.diff");

    let mut patch = PatchSet::new();
    patch.parse(&buf).unwrap();

    assert_eq!(3, patch.len());

    let added_files = patch.added_files();
    assert_eq!(1, added_files.len());
    assert_eq!("added_file", added_files[0].path());
    assert_eq!(4, added_files[0].added());
    assert_eq!(0, added_files[0].removed());

    let removed_files = patch.removed_files();
    assert_eq!(1, removed_files.len());
    assert_eq!("removed_file", removed_files[0].path());
    assert_eq!(0, removed_files[0].added());
    assert_eq!(3, removed_files[0].removed());

    let modified_files = patch.modified_files();
    assert_eq!(1, modified_files.len());
    assert_eq!("modified_file", modified_files[0].path());
    assert_eq!(3, modified_files[0].added());
    assert_eq!(1, modified_files[0].removed());
}

#[test]
fn test_parse_bzr_diff() {
    let buf = include_str!("fixtures/bzr.diff");

    let mut patch = PatchSet::new();
    patch.parse(&buf).unwrap();

    assert_eq!(3, patch.len());

    let added_files = patch.added_files();
    assert_eq!(1, added_files.len());
    assert_eq!("added_file", added_files[0].path());
    assert_eq!(4, added_files[0].added());
    assert_eq!(0, added_files[0].removed());

    let removed_files = patch.removed_files();
    assert_eq!(1, removed_files.len());
    assert_eq!("removed_file", removed_files[0].path());
    assert_eq!(0, removed_files[0].added());
    assert_eq!(3, removed_files[0].removed());

    let modified_files = patch.modified_files();
    assert_eq!(1, modified_files.len());
    assert_eq!("modified_file", modified_files[0].path());
    assert_eq!(3, modified_files[0].added());
    assert_eq!(1, modified_files[0].removed());
}

#[test]
fn test_parse_hg_diff() {
    let buf = include_str!("fixtures/hg.diff");

    let mut patch = PatchSet::new();
    patch.parse(&buf).unwrap();

    assert_eq!(3, patch.len());

    let added_files = patch.added_files();
    assert_eq!(1, added_files.len());
    assert_eq!("added_file", added_files[0].path());
    assert_eq!(4, added_files[0].added());
    assert_eq!(0, added_files[0].removed());

    let removed_files = patch.removed_files();
    assert_eq!(1, removed_files.len());
    assert_eq!("removed_file", removed_files[0].path());
    assert_eq!(0, removed_files[0].added());
    assert_eq!(3, removed_files[0].removed());

    let modified_files = patch.modified_files();
    assert_eq!(1, modified_files.len());
    assert_eq!("modified_file", modified_files[0].path());
    assert_eq!(3, modified_files[0].added());
    assert_eq!(1, modified_files[0].removed());
}

#[test]
fn test_parse_svn_diff() {
    let buf = include_str!("fixtures/svn.diff");

    let mut patch = PatchSet::new();
    patch.parse(&buf).unwrap();

    assert_eq!(3, patch.len());

    let added_files = patch.added_files();
    assert_eq!(1, added_files.len());
    assert_eq!("added_file", added_files[0].path());
    assert_eq!(4, added_files[0].added());
    assert_eq!(0, added_files[0].removed());

    let removed_files = patch.removed_files();
    assert_eq!(1, removed_files.len());
    assert_eq!("removed_file", removed_files[0].path());
    assert_eq!(0, removed_files[0].added());
    assert_eq!(3, removed_files[0].removed());

    let modified_files = patch.modified_files();
    assert_eq!(1, modified_files.len());
    assert_eq!("modified_file", modified_files[0].path());
    assert_eq!(3, modified_files[0].added());
    assert_eq!(1, modified_files[0].removed());
}

#[test]
fn test_parse_line_numbers() {
    let buf = include_str!("fixtures/sample0.diff");

    let mut patch = PatchSet::new();
    patch.parse(&buf).unwrap();

    let mut target_line_nos = vec![];
    let mut source_line_nos = vec![];
    let mut diff_line_nos = vec![];

    for diff_file in patch {
        for hunk in diff_file {
            for line in hunk {
                source_line_nos.push(line.source_line_no.clone());
                target_line_nos.push(line.target_line_no.clone());
                diff_line_nos.push(line.diff_line_no);
            }
        }
    }

    let expected_target_line_nos = vec![
        // File: 1, Hunk: 1
        Some(1),
        Some(2),
        Some(3),
        Some(4),
        Some(5),
        Some(6),
        Some(7),
        Some(8),
        Some(9),
        // File: 1, Hunk: 2
        Some(11),
        Some(12),
        Some(13),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(14),
        Some(15),
        Some(16),
        None,
        Some(17),
        Some(18),
        Some(19),
        Some(20),
        // File: 1, Hunk: 3
        Some(22),
        Some(23),
        Some(24),
        Some(25),
        Some(26),
        Some(27),
        Some(28),
        // File: 2, Hunk 1
        Some(1),
        Some(2),
        Some(3),
        Some(4),
        Some(5),
        Some(6),
        Some(7),
        Some(8),
        Some(9),
        // File: 3, Hunk 1
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ];
    let expected_source_line_nos = vec![
        // File: 1, Hunk: 1
        None,
        None,
        None,
        None,
        None,
        None,
        Some(1),
        Some(2),
        Some(3),
        // File: 1, Hunk: 2
        Some(5),
        Some(6),
        Some(7),
        Some(8),
        Some(9),
        Some(10),
        Some(11),
        Some(12),
        Some(13),
        Some(14),
        None,
        Some(15),
        Some(16),
        Some(17),
        None,
        Some(18),
        Some(19),
        Some(20),
        // File: 1, Hunk: 3
        Some(22),
        Some(23),
        Some(24),
        None,
        None,
        None,
        None,
        // File: 2, Hunk 1
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        // File: 3, Hunk 1
        Some(1),
        Some(2),
        Some(3),
        Some(4),
        Some(5),
        Some(6),
        Some(7),
        Some(8),
        Some(9),
    ];
    let expected_diff_line_nos = vec![
        // File: 1, Hunk: 1
        4, 5, 6, 7, 8, 9, 10, 11, 12, // File: 1, Hunk: 2
        14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
        // File: 1, Hunk: 3
        33, 34, 35, 36, 37, 38, 39, // File: 2, Hunk 1
        43, 44, 45, 46, 47, 48, 49, 50, 51, // File: 3, Hunk 1
        55, 56, 57, 58, 59, 60, 61, 62, 63,
    ];

    assert_eq!(expected_source_line_nos, source_line_nos);
    assert_eq!(expected_target_line_nos, target_line_nos);
    assert_eq!(expected_diff_line_nos, diff_line_nos);
}

#[cfg(feature = "encoding")]
#[test]
fn test_parse_from_encoding() {
    let buf = include_bytes!("fixtures/sample3.diff");

    let mut patch = PatchSet::from_encoding("utf-8");
    patch.parse_bytes(buf).unwrap();

    assert_eq!(3, patch.len());
    assert_eq!("holá mundo!", patch[0][0][1].value);
}

#[test]
fn test_single_line_diff() {
    {
        let buf = include_str!("fixtures/sample4.diff");

        let mut patch = PatchSet::new();
        patch.parse(&buf).unwrap();

        assert_eq!(1, patch.len());

        let added_files = patch.added_files();
        assert_eq!(1, added_files.len());
        assert_eq!("sample.txt", added_files[0].path());
        assert_eq!(1, added_files[0].added());
        assert_eq!(0, added_files[0].removed());

        let hunks = added_files[0].hunks();
        assert_eq!(1, hunks.len());
        assert_eq!(0, hunks[0].source_start);
        assert_eq!(0, hunks[0].source_length);
        assert_eq!(1, hunks[0].target_start);
        assert_eq!(1, hunks[0].target_length);
    }
    {
        let buf = include_str!("fixtures/sample5.diff");

        let mut patch = PatchSet::new();
        patch.parse(&buf).unwrap();

        assert_eq!(1, patch.len());

        let removed_files = patch.removed_files();
        assert_eq!(1, removed_files.len());
        assert_eq!("sample.txt", removed_files[0].path());
        assert_eq!(0, removed_files[0].added());
        assert_eq!(1, removed_files[0].removed());
    }
}

#[test]
fn test_single_line_diff_with_trailer() {
    let buf = include_str!("fixtures/sample4-plus.diff");

    let mut patch = PatchSet::new();
    patch.parse(&buf).unwrap();

    assert_eq!(3, patch.len());

    let added_files = patch.added_files();
    assert_eq!(3, added_files.len());
    assert_eq!("sample.txt", added_files[0].path());
    assert_eq!(1, added_files[0].added());
    assert_eq!(0, added_files[0].removed());

    assert_eq!("sample2.txt", added_files[1].path());
    assert_eq!(1, added_files[1].added());
    assert_eq!(0, added_files[1].removed());

    assert_eq!("sample3.txt", added_files[2].path());
    assert_eq!(1, added_files[2].added());
    assert_eq!(0, added_files[2].removed());
}

#[test]
fn test_parse_patchset_from_str() {
    let buf = include_str!("fixtures/sample0.diff");

    let patch: PatchSet = buf.parse().unwrap();

    // three file in the patch
    assert_eq!(3, patch.len());
    // three hunks
    assert_eq!(3, patch[0].len());
}
