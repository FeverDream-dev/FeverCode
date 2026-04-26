Contributing to FeverCode

- Fork this repository and create a feature/bug branch from main.
- Use conventional commits for your messages (e.g., feat(provider): add streaming support).
- Open PRs against main. Ensure CI passes (cargo test, cargo fmt, cargo clippy) and is reviewed.
- Code style: run cargo fmt --all and cargo clippy --all-targets --all-features.
- Testing: run cargo test. Include unit tests and integration tests where appropriate.
- Safety philosophy: changes stay within the workspace, avoid executing untrusted commands, and keep patches focused and minimal.
