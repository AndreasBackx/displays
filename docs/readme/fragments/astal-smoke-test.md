## Smoke Testing

The repository includes a fake-backend GJS smoke test at `examples/test.js`.
It expects a typelib built with the `faked` feature.

```sh
env GI_TYPELIB_PATH="$PWD/build-faked" gjs -m "$PWD/examples/test.js"
```
