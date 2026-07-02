/// Pipe stages per §16 pipe_stage: bare callee, call args, method chains,
/// turbofish, and the stage-trailing `?` that wraps the accumulated pipe
/// application (§5.7 — `x |> f?` is `(x |> f)?`).
fn pipeline(input: String, checker: Checker) -> Result<i64, ParseError> {
    let trimmed = input |> trim;
    let n = trimmed |> parse::<i64>?;
    let m = n |> add(5) |> add(20);
    let v = m |> checker.validate(3)?;
    let chained = v |> step_one? |> step_two?;
    let arith = (chained |> double) + offset(1);
    arith |> finish
}
