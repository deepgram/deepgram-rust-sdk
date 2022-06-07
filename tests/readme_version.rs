use std::fs;

/// Checks that README.md contains the correct version
#[test]
fn readme_version() {
    let readme = fs::read_to_string("README.md").unwrap();

    let target = concat!(
        env!("CARGO_PKG_NAME"),
        " = \"",
        env!("CARGO_PKG_VERSION"),
        "\""
    );

    assert!(readme.contains(target));
}
