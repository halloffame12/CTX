# ctx Homebrew tap

The live tap lives at **https://github.com/halloffame12/homebrew-ctx** (repo `halloffame12/homebrew-ctx`).

To use:
```sh
brew tap halloffame12/ctx
brew install ctx
```

This directory (`packaging/homebrew/tap/`) is the development copy of the tap's `Formula/ctx.rb`.
The published formula at `halloffame12/homebrew-ctx/Formula/ctx.rb` is kept in sync by hand.

To update the formula after a release:
1. Build and upload release binaries (vX.Y.Z).
2. Get the SHA-256 hashes: `scripts/generate-checksums.sh`.
3. Edit `Formula/ctx.rb` with the new version + sha256 values.
4. Commit and push to `halloffame12/homebrew-ctx` (branch `main`).
