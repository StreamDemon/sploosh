# Example: CLI Tool

> A command-line application using `std::env` and `std::fs`.

## main.sp

```sploosh
use std::env;
use std::fs;
use std::json;

@error
enum CliError {
    MissingArgument { name: String },
    FileNotFound { path: String },
    ParseError(json::JsonError),
}

fn main() -> Result<(), CliError> {
    let args = env::args();

    if args.len() < 2 {
        return Err(CliError::MissingArgument { name: "config_path".into() });
    }

    let config_path = &args[1];
    let content = fs::read(config_path)
        .context(format("failed to read {}", config_path))?;

    let config: Config = json::parse(&content)?;

    match config.command.as_str() {
        "build" => run_build(config),
        "test" => run_tests(config),
        "deploy" => run_deploy(config),
        _ => {
            print(format("Unknown command: {}", config.command));
            Ok(())
        }
    }
}

struct Config {
    command: String,
    target: String,
    verbose: bool,
}

fn run_build(config: Config) -> Result<(), CliError> {
    if config.verbose {
        print(format("Building for target: {}", config.target));
    }
    // ...
    Ok(())
}

fn run_tests(config: Config) -> Result<(), CliError> { Ok(()) }
fn run_deploy(config: Config) -> Result<(), CliError> { Ok(()) }
```

## Key Patterns

- **`@error` derive** for clean error enum with auto-generated `From` impls
- **`env::args()`** for CLI arguments
- **`.context()`** for adding error context
- **Match on strings** for command dispatch
- **Result-based flow** throughout -- no exceptions
