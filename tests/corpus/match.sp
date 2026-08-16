/// Match shapes from §5.2/§3.7/§8.10.1: guards, tuple/variant/struct
/// destructuring, or-patterns, rest `..`, and mixed arm bodies. Expression
/// bodies carry the mandatory trailing comma, block bodies none (§16
/// match_arm). §5.2's `Err(e) => return Err(e),` arm is written as a block
/// body here — §16 has no `return` expression alternative (see #89).
enum AppError {
    NotFound,
    Timeout { after: i64 },
    Other { message: String },
}

enum Role {
    Admin,
    Editor { level: i64 },
    Viewer,
    Guest,
    Mod { name: String },
}

enum Shape {
    Circle { radius: i64 },
    Rect { width: i64, height: i64 },
    Point,
}

fn describe(result: Result<i64, AppError>) -> String {
    match result {
        Ok(value) => format("ok: {}", value),
        Err(AppError::NotFound) => "missing".into(),
        Err(AppError::Timeout { after }) => format("timeout after {}", after),
        Err(e) => { return format("error: {}", e); }
    }
}

// §5.2: guards sit between the pattern and `=>`.
fn bracket(age: i64) -> String {
    match age {
        n if n < 13 => "child",
        n if n < 20 => "teen",
        n if n < 65 => "adult",
        _ => "senior",
    }
}

// §5.2: tuple destructuring with literal and binding elements.
fn axis(point: (i64, i64)) -> String {
    match point {
        (0, 0) => "origin",
        (x, 0) => format("{} on x-axis", x),
        (0, y) => format("{} on y-axis", y),
        (x, y) => format("({}, {})", x, y),
    }
}

// §5.2's `match self` example, in free-function form: struct variants and a
// unit variant, matched over a reference.
fn shape_label(shape: &Shape) -> String {
    match shape {
        Shape::Circle { radius } => format("circle r={}", radius),
        Shape::Rect { width, height } => format("rect {}x{}", width, height),
        Shape::Point => "point".into(),
    }
}

fn role_label(role: Role) -> String {
    match role {
        Role::Admin => "admin",
        Role::Editor { level } => format("editor-{}", level),
        Role::Viewer | Role::Guest => "limited",
        Role::Mod { name, .. } => name,
    }
}

// §3.10: string literal patterns; a match may head a pipe (§5.6) and an arm
// body may itself be a pipe with stage-`?` (§5.7).
fn kind_score(kind: String) -> i64 {
    match kind {
        "circle" => 1,
        _ => 2,
    } |> double
}

fn parse_or_zero(value: String) -> i64 {
    match classify(value) {
        Ok(digits) => digits |> parse::<i64>?,
        Err(_) => 0,
    }
}
