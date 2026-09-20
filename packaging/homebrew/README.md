# Homebrew tap

The canonical formula is [`Formula/ipclc.rb`](../../Formula/ipclc.rb). Copy that single file to the `psvmcc/homebrew-tap` tap repository.

To install after publishing that tap:

```console
brew tap psvmcc/tap
brew install psvmcc/tap/ipclc
```

The repository is named `homebrew-tap`; Homebrew exposes it as the tap `psvmcc/tap`.

After all CI gates and release builds pass, `.github/workflows/ci.yml` updates the tap automatically with platform-specific release URLs and SHA-256 values. Create the repository secret `HOMEBREW_TAP_TOKEN` with write access to `psvmcc/homebrew-tap`. Homebrew downloads a prebuilt binary and does not require Rust or Cargo for installation.
