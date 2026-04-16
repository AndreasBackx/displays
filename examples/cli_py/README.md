# Python CLI Example

This is a small Click-based CLI example for the local `displays` Python bindings.

## Usage

```shell
uv sync
uv run displays-cli query
uv run displays-cli apply --name "Dell U2720Q" --brightness 50
```

The project uses the local `displays_py` package through `tool.uv.sources`, so it always runs against the bindings in this repository.
