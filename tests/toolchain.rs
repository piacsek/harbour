use std::path::Path;

#[test]
fn rust_version_is_the_major_minor_of_the_pinned_toolchain() {
    assert_eq!(rust_version(), major_minor(&channel()));
}

#[test]
fn the_flake_builds_with_the_toolchain_the_file_names() {
    assert!(read("flake.nix").contains("fromRustupToolchainFile ./rust-toolchain.toml"));
}

#[test]
fn the_flake_leaves_out_the_components_the_build_never_runs() {
    assert!(read("flake.nix").contains(
        "(rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override { extensions = [ ]; }"
    ));
}

#[test]
fn the_flake_names_no_rust_version_of_its_own() {
    let literals = version_literals(&read("flake.nix"));

    assert!(literals.is_empty(), "{literals:?}");
}

#[test]
fn a_rust_version_other_than_the_pinned_one_counts_as_a_literal() {
    let literals = version_literals("toolchain = rust-bin.stable.\"1.99.1\".default;");

    assert_eq!(literals.len(), 1);
    assert_eq!(literals[0], "1.99.1");
}

#[test]
fn a_major_minor_counts_as_a_literal() {
    let literals = version_literals("rust-bin.stable.\"1.98\".minimal");

    assert_eq!(literals.len(), 1);
    assert_eq!(literals[0], "1.98");
}

#[test]
fn a_nixpkgs_release_branch_is_no_rust_version() {
    let literals = version_literals("nixpkgs.url = \"github:NixOS/nixpkgs/nixos-26.11\";");

    assert!(literals.is_empty(), "{literals:?}");
}

#[test]
fn the_flake_leaves_the_rust_platform_of_nixpkgs_alone() {
    assert!(!uses_nixpkgs_rust(&read("flake.nix")));
}

#[test]
fn building_with_the_rust_platform_of_nixpkgs_counts_as_using_its_rust() {
    assert!(uses_nixpkgs_rust("pkgs.rustPlatform.buildRustPackage {"));
}

#[test]
fn the_toolchain_leaves_out_the_docs_the_flake_would_unpack() {
    assert_eq!(
        toolchain_file().get("profile").and_then(|p| p.as_str()),
        Some("minimal")
    );
}

fn version_literals(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_ascii_digit() && c != '.')
        .map(|run| run.trim_matches('.'))
        .filter(|run| run.starts_with("1."))
        .filter(|run| run.split('.').count() > 1 && run.split('.').all(|part| !part.is_empty()))
        .map(str::to_string)
        .collect()
}

fn uses_nixpkgs_rust(text: &str) -> bool {
    text.contains("pkgs.rustPlatform")
}

fn read(path: &str) -> String {
    let full = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    std::fs::read_to_string(&full).unwrap_or_else(|e| panic!("{}: {e}", full.display()))
}

fn toolchain_file() -> toml::Table {
    let file: toml::Table = toml::from_str(&read("rust-toolchain.toml")).unwrap();
    file["toolchain"].as_table().unwrap().clone()
}

fn channel() -> String {
    toolchain_file()["channel"].as_str().unwrap().to_string()
}

fn rust_version() -> String {
    let manifest: toml::Table = toml::from_str(&read("Cargo.toml")).unwrap();
    manifest["package"]["rust-version"]
        .as_str()
        .unwrap()
        .to_string()
}

fn major_minor(version: &str) -> String {
    version.splitn(3, '.').take(2).collect::<Vec<_>>().join(".")
}
