use sploosh_parser::parse_program;

/// Parses every `.sp` fixture in `tests/corpus/` at the repo root. Fixtures
/// are discovered, not listed, so a new file cannot be silently skipped
/// (crates/AGENTS.md: corpus tests for every accepted grammar shape).
#[test]
fn parses_corpus_files() {
    let corpus_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus");
    let mut paths: Vec<_> = std::fs::read_dir(&corpus_dir)
        .unwrap_or_else(|err| panic!("{}: {err}", corpus_dir.display()))
        .map(|entry| entry.expect("corpus dir entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "sp"))
        .collect();
    paths.sort();
    assert!(
        !paths.is_empty(),
        "no .sp fixtures found in {}",
        corpus_dir.display()
    );
    for path in paths {
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        parse_program(&source).unwrap_or_else(|errors| panic!("{}: {errors:#?}", path.display()));
    }
}
