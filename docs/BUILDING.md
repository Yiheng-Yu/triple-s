# Documentation builds

Read the Docs installs the wheel matching `Cargo.toml`; it never compiles Rust.
Push-triggered builds cancel if the wheel hasn't been published. The release
workflow triggers `release` and `latest` docs after PyPI publication succeeds.
Both Read the Docs versions should point to the `release` branch.

The trigger uses `READTHEDOCS_TOKEN` from the GitHub `release-build` environment.
To retry without republishing, run **Build documentation** in GitHub Actions.

For a local build with the package already installed:

```sh
python -m sphinx -M html docs docs/_build -W
```
