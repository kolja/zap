{ pkgs ? import <nixpkgs> {} }:

let
  inherit (pkgs) lib rustPlatform fetchFromGitHub;
  version = "0.1.5";
in
rustPlatform.buildRustPackage {
  pname = "zap";
  inherit version;

  src = fetchFromGitHub {
    owner = "kolja";
    repo = "zap";
    rev = "v${version}";
    hash = "sha256-uzGJnLjJXhxe4HWFL5FJCOXXAp1uJdtDEGTPlRGnJ64=";
  };

  # New cargo hash after downgrading edition
  cargoHash = "sha256-q3mQ/pQp3yX9gJGf8zC+t4fE88o2/nB8bU8cO+3D75w=";

  meta = with lib; {
    description = "`touch`, but with templates!";
    homepage = "https://github.com/kolja/zap";
    license = licenses.mit;
    maintainers = with maintainers; [ "kolja" ];
    platforms = platforms.all;
  };
}
