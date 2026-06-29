use sploosh_parser::parse_program;

#[test]
fn parses_corpus_files() {
    for path in ["tests/corpus/basic.sp", "tests/corpus/actor.sp"] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path);
        let source = std::fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("{}: {err}", path.display());
        });
        parse_program(&source).unwrap_or_else(|errors| panic!("{}: {errors:#?}", path.display()));
    }
}
