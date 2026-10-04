//! The version of the product in its four parts.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

// realises FR-224
/// Yields the version of *product* in its four parts from the version of the package.
///
/// * `<major>.<minor>.<patch>` yields `<major>.<minor>.<patch>.0`.
/// * `<major>.<minor>.<patch>+<build>` yields `<major>.<minor>.<patch>.<build>`.
/// * Each part is a run of decimal digits; a version of another form, which 'Cargo.toml' of
///   *product* does not carry, is yielded as it is.
///
/// # Arguments
/// * `package_version` - the version of the package, as `env!("CARGO_PKG_VERSION")` yields it
pub fn version_of_package(package_version: &str) -> String {
    let (release, build) = package_version
        .split_once('+')
        .unwrap_or((package_version, "0"));
    let parts: Vec<&str> = release.split('.').collect();
    let is_number = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    if parts.len() == 3 && parts.iter().all(|part| is_number(part)) && is_number(build) {
        format!("{release}.{build}")
    } else {
        package_version.to_owned()
    }
}

/*  * validated        : ✅
 * completeness     : ✅ three parts, three parts with a build part, the forms of another kind
 * independence     : ✅
 * edge cases       : ✅ build part 0, an empty part, two parts, four parts, a pre-release
 * conforms to doc  : ✅
 * covers bridge    : Workbench::version */
#[cfg(test)]
mod tests {
    use super::*;

    /// The version of the topmost version heading of 'CHANGELOG.md', released or not.
    fn topmost_changelog_version() -> Option<String> {
        let changelog =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/CHANGELOG.md"))
                .expect("the package holds 'CHANGELOG.md' at its root");
        changelog.lines().find_map(|line| {
            let heading = line
                .strip_prefix("### [")
                .or_else(|| line.strip_prefix("## ["))?;
            let (version, _) = heading.split_once(']')?;
            version
                .starts_with(|first: char| first.is_ascii_digit())
                .then(|| version.to_owned())
        })
    }

    #[test]
    fn three_parts_yield_build_part_zero() {
        assert_eq!(version_of_package("0.16.1"), "0.16.1.0");
    }

    #[test]
    fn build_metadata_yields_build_part() {
        assert_eq!(version_of_package("0.6.0+2"), "0.6.0.2");
    }

    #[test]
    fn build_metadata_zero_yields_build_part_zero() {
        assert_eq!(version_of_package("1.0.0+0"), "1.0.0.0");
    }

    #[test]
    fn pre_release_is_yielded_as_it_is() {
        assert_eq!(version_of_package("0.16.1-beta"), "0.16.1-beta");
    }

    #[test]
    fn build_metadata_of_letters_is_yielded_as_it_is() {
        assert_eq!(version_of_package("0.16.1+abc"), "0.16.1+abc");
    }

    #[test]
    fn two_parts_are_yielded_as_they_are() {
        assert_eq!(version_of_package("0.16"), "0.16");
    }

    #[test]
    fn four_parts_are_yielded_as_they_are() {
        assert_eq!(version_of_package("0.16.1.0"), "0.16.1.0");
    }

    #[test]
    fn empty_part_is_yielded_as_it_is() {
        assert_eq!(version_of_package("0..1+"), "0..1+");
    }

    #[test]
    fn package_version_is_the_topmost_changelog_version() {
        assert_eq!(
            Some(version_of_package(env!("CARGO_PKG_VERSION"))),
            topmost_changelog_version()
        );
    }
}
