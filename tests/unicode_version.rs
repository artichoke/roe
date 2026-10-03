#[test]
fn public_unicode_version_matches_bundled_data() {
    let (major, minor, patch) = roe::UNICODE_VERSION;
    let marker = format!("final data files for version {major}.{minor}.{patch}");
    assert!(include_str!("../generated/ucd/ReadMe.txt").contains(&marker));
}
