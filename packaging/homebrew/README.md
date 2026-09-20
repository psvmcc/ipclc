# Homebrew tap

The canonical formula is [`Formula/ipclc.rb`](../../Formula/ipclc.rb). Copy that single file to the `psvmcc/homebrew-ipclc` tap repository.

To install after publishing that tap:

```console
brew tap psvmcc/ipclc
brew install ipclc
```

The repository name must be `homebrew-ipclc`; Homebrew exposes it as the tap `psvmcc/ipclc`.

After all CI gates and release builds pass, `.github/workflows/ci.yml` updates the tap automatically with both the tag and immutable commit revision. Create the repository secret `HOMEBREW_TAP_TOKEN` with write access to `psvmcc/homebrew-ipclc`. The formula builds from tagged source with Cargo and does not run the project test suite during installation.
