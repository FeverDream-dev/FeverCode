class Fever < Formula
  desc "Terminal AI coding agent — Rust-powered TUI with 53 LLM providers"
  homepage "https://github.com/FeverDream-dev/FeverCode"
  url "https://github.com/FeverDream-dev/FeverCode/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER_UPDATE_AFTER_FIRST_RELEASE"
  license "BSL-1.1"
  head "https://github.com/FeverDream-dev/FeverCode.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "build", "--release", "--bin", "fever", "--manifest-path", "fevercode_starter/Cargo.toml"
    bin.install "fevercode_starter/target/release/fever"
    bin.install_symlink "fever" => "fevercode"
  end

  test do
    assert_match "fever", shell_output("#{bin}/fever --version")
  end
end
