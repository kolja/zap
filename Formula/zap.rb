class Zap < Formula
  desc "touch, but with templates"
  homepage "https://github.com/kolja/zap"
  license "MIT"

  VERSION = "v0.1.2"

  # to figure out the latest sha sum (over in the kolja/zap repo) run:
  # > cargo make list-sha
  SHA256_DARWIN_ARM = "46b1d6978d61f5bb420bc4dabfeb3f3f800b3a588a710d6619813630f18115b3"
  SHA256_LINUX_X86 = "7c24c9caad9d2dc5c473376866000a84e68b46795ae20e03801f901544757cfc"

  BASE_URL = "https://github.com/kolja/zap/releases/download"

  version VERSION

  if OS.mac?
    url "#{BASE_URL}/#{VERSION}/zap-aarch64-apple-darwin.tar.gz"
    sha256 SHA256_DARWIN_ARM
  elsif OS.linux?
    url "#{BASE_URL}/#{VERSION}/zap-x86_64-unknown-linux-musl.tar.gz"
    sha256 SHA256_LINUX_X86
  end

  def install
    bin.install "zap"
  end

  test do
    system "#{bin}/zap", "--version"
  end
end
