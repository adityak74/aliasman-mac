# Template — placeholders filled in by scripts/release.sh
# Do not use this file directly with Homebrew.
class Aliasman < Formula
  desc "Terminal alias manager for macOS with semantic search and Claude Code integration"
  homepage "https://github.com/adityak74/aliasman-mac"
  version "{{VERSION}}"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/adityak74/aliasman-mac/releases/download/v{{VERSION}}/aliasman-{{VERSION}}-aarch64-apple-darwin.tar.gz"
      sha256 "{{ARM_SHA256}}"
    else
      url "https://github.com/adityak74/aliasman-mac/releases/download/v{{VERSION}}/aliasman-{{VERSION}}-x86_64-apple-darwin.tar.gz"
      sha256 "{{X86_SHA256}}"
    end
  end

  def install
    bin.install "aliasman"
  end

  def caveats
    <<~EOS
      After installation, initialize aliasman for your shell:

        aliasman init

      This auto-detects your shell (zsh/bash), imports existing aliases,
      and injects a managed block into your shell config.

      For semantic search (optional), install Ollama and pull the embedding model:
        brew install ollama
        ollama pull nomic-embed-text

      For Claude Code integration:
        aliasman hook install
    EOS
  end

  test do
    assert_match "aliasman", shell_output("#{bin}/aliasman --version")
  end
end
