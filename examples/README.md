# Fever Code Examples

This directory contains examples of using Fever Code.

## Example Configuration

### Minimal Configuration

```toml
[defaults]
provider = "ollama"
model = "llama2"
temperature = 0.7
max_tokens = 4096

[providers.ollama]
enabled = true
base_url = "http://localhost:11434"
```

### OpenAI Configuration

```toml
[defaults]
provider = "openai"
model = "gpt-4o"
temperature = 0.7
max_tokens = 4096

[providers.openai]
enabled = true
api_key = "sk-your-api-key-here"
```

### Anthropic Configuration

```toml
[defaults]
provider = "anthropic"
model = "claude-3-opus-20240229"
temperature = 0.7
max_tokens = 4096

[providers.anthropic]
enabled = true
api_key = "your-anthropic-api-key"
```

## Example Workflows

### Creating a New Rust Project

1. Start Fever Code: `fever code`
2. Type: "Create a new Rust project with clap for CLI argument parsing"
3. Fever Code will:
   - Create the project structure
   - Add necessary dependencies
   - Implement the CLI
   - Write initial code

### Debugging a Rust Error

1. Copy the error message
2. Paste into Fever Code: "I'm getting this error: [paste error]"
3. Fever Code will:
   - Analyze the error
   - Explain the cause
   - Provide a fix
   - Apply the fix if requested

### Code Review

1. Fever Code: "Review the current file for best practices"
2. Fever Code will:
   - Analyze code quality
   - Check for potential issues
   - Suggest improvements
   - Refactor if requested

## Example Usage

See the main README.md for more information on using Fever Code.
