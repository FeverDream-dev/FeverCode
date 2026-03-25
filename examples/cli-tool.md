# Example: Creating a Simple CLI Tool

This example demonstrates how Fever Code can help create a simple CLI tool.

## What You'll Learn

- How Fever Code creates project structure
- How it adds dependencies
- How it implements CLI argument parsing
- How it writes idiomatic Rust code

## Steps

1. Start Fever Code:
   ```bash
   fever code
   ```

2. In the chat, type:
   ```
   Create a simple CLI tool called "greet" that:
   - Takes a name as an argument
   - Greets the person with a friendly message
   - Has a --uppercase flag to print in uppercase
   - Uses clap for argument parsing
   - Includes proper error handling
   ```

3. Fever Code will:
   - Create the project structure
   - Add clap dependency
   - Implement the CLI
   - Write the main.rs file
   - Add tests

4. Test the tool:
   ```bash
   cargo run -- greet --name "World"
   cargo run -- greet --name "World" --uppercase
   ```

## Expected Outcome

Fever Code should create a working CLI tool with:
- Proper argument parsing
- Help text
- Error handling
- Tests

## Manual Code

If you want to see what Fever Code generates, here's what the final code might look like:

```rust
use clap::Parser;

/// A simple greeting CLI tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Print greeting in uppercase
    #[arg(short, long, default_value_t = false)]
    uppercase: bool,
}

fn main() {
    let args = Args::parse();

    let greeting = format!("Hello, {}!", args.name);
    let output = if args.uppercase {
        greeting.to_uppercase()
    } else {
        greeting
    };

    println!("{}", output);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lowercase() {
        let args = Args::parse_from(&["test", "--name", "world"]);
        let greeting = format!("Hello, {}!", args.name);
        assert_eq!(greeting, "Hello, world!");
    }

    #[test]
    fn test_uppercase() {
        let args = Args::parse_from(&["test", "--name", "world", "--uppercase"]);
        let greeting = format!("Hello, {}!", args.name);
        assert_eq!(greeting.to_uppercase(), "HELLO, WORLD!");
    }
}
```
