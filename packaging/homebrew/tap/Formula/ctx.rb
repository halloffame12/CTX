# Homebrew formula for ctx — installed from prebuilt release binaries.
#
# Tap repository layout:
#   <tap>/
#     Formula/
#       ctx.rb            <- this file
#
# Usage:
#   brew tap halloffame12/CTX
#   brew install ctx
#
# sha256 placeholders are regenerated per release by
# scripts/update-homebrew.sh.
class Ctx < Formula
  desc "Codebase intelligence and context engine for AI coding agents"
  homepage "https://github.com/halloffame12/CTX"
  version "0.1.0"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/halloffame12/CTX/releases/download/v0.1.0/ctx-macos-aarch64"
    sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/halloffame12/CTX/releases/download/v0.1.0/ctx-macos-x86_64"
    sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  elsif OS.linux? && Hardware::CPU.arm?
    url "https://github.com/halloffame12/CTX/releases/download/v0.1.0/ctx-linux-aarch64"
    sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  elsif OS.linux? && Hardware::CPU.intel?
    url "https://github.com/halloffame12/CTX/releases/download/v0.1.0/ctx-linux-x86_64"
    sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  else
    odie "ctx: unsupported platform — only macOS (Intel/ARM) and Linux (Intel/ARM) are supported"
  end

  def install
    # The downloaded artifact is a single named file; strip the prefix.
    bin.install Dir["ctx-*"].first => "ctx"
  end

  test do
    assert_match "ctx #{version}", shell_output("#{bin}/ctx --version")
  end
end