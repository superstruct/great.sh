### Bug Fixes

- **platform**: Fixed false WSL detection when running inside Docker/Podman containers on WSL2 hosts. The CLI now correctly identifies the environment as Linux rather than WSL, preventing attempts at WSL-specific actions (font copying, cmd.exe invocation) that fail inside containers.
