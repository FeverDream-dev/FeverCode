# FeverCode MCP Support — Transport and Status

Current status: Implemented but not wired to the agent loop. The MCP (Message Control Protocol) client exists and can operate with stdio transport.

Config format
- A simple YAML/JSON provider map is used to configure MCP transports.

Example (YAML)

```
mcp:
  transport: stdio
  stdio:
    input_path: /tmp/mcp_in
    output_path: /tmp/mcp_out
    echo: true
```

Transport: stdio
- FeverCode communicates with MCP using standard input and output streams. This is designed for local IPC and testing.

Current status and future work
- Implemented: MCP client skeleton and stdio transport exist.
- Wired to agent loop: Planned. Will enable real-time MCP messaging to trigger actions.
- Other transports (socket, HTTP) and richer protocol features: Planned.
