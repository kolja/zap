class Zap < Formula
  desc "touch, but with templates"
  homepage "https://github.com/kolja/zap"
  license "MIT"

  VERSION = "v0.1.3"

  # to figure out the latest sha sum (over in the kolja/zap repo) run:
  # > cargo make list-sha
  SHA256_DARWIN_ARM = "3b5413859d0e120605c4daeb12aad6fa58cac33dc809e0d7083222ecb8a45eae"
  SHA256_LINUX_X86 = "299116b265de7e1ae9ac913518b42d6a414710ed9a8f5d323a36d2614f6b1e3b"

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
