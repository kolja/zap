class Zap < Formula
  desc "touch, but with templates"
  homepage "https://github.com/kolja/zap"
  license "MIT"

  VERSION = "v0.1.1"

  # to figure out the latest sha sum (over in the kolja/zap repo) run:
  # > cargo make list-sha
  SHA256_DARWIN_ARM = "55ae6727dd336632d8fc6072f8e55b2a93b15511e28d5936ce7108339be718c5"
  SHA256_LINUX_X86 = "0b1061ea0dac7adeaf04fd0a05084b80893550a2a06cebe8d9900a23f406ece9"

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
