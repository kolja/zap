# default.nix for v0.1.4
{ pkgs ? import <nixpkgs> {} }:

let
  inherit (pkgs) lib rustPlatform fetchFromGitHub;
  version = "0.1.4";
in
rustPlatform.buildRustPackage {
  pname = "zap";
  inherit version;

  src = fetchFromGitHub {
    owner = "kolja";
    repo = "zap";
    rev = "v${version}";
    # New hash from `nix-prefetch-url --unpack`
    hash = "sha256-yourNewSourceHashHere";
  };

  # New hash from the failed build output
  cargoHash = "sha256-yourNewCargoHashHere";

  meta = with lib; {
    description = "`touch`, but with templates!";
    homepage = "https://github.com/kolja/zap";
    license = licenses.mit;
    maintainers = with maintainers; [ "kolja" ];
    platforms = platforms.all;
  };
}
