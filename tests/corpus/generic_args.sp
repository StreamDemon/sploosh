/// Generic type arguments (§16 type_arg = type | IDENT "=" type): non-ident-
/// headed argument types — references, tuples, arrays — alongside associated-
/// type bindings whose value types are themselves non-ident-headed.
struct Grid {
    names: Vec<&str>,
    cells: Vec<(i64, i64)>,
    rows: Vec<[u8; 4]>,
    lookup: Map<String, Vec<&str>>,
}

struct Streams {
    it: Box<dyn Iter<Item = Vec<&str>>>,
}

fn columns(grid: &Grid) -> Vec<(i64, i64)> {
    grid.cells
}
