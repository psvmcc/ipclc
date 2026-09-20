class Ipclc < Formula
  desc "Cross-platform IPv4 and IPv6 network calculator"
  homepage "https://github.com/psvmcc/ipclc"
  url "https://github.com/psvmcc/ipclc.git", using: :git, tag: "v0.1.0"
  version "0.1.0"
  license "MIT"

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  test do
    assert_match "192.168.1.0", shell_output("#{bin}/ipclc 192.168.1.42/24")
    assert_match "2001:db8::", shell_output("#{bin}/ipclc 2001:db8::1/64")
  end
end
