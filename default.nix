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
    hash = "sha256-uL5FqWRDnFuhWyblzxbNDBfybGYFcPhX/e5aXqQ8d0A=";
  };

  cargoHash = "sha256-YUQjwldX3VZqa2vF4D2hTF6wYzeKOoWTi0lu8i1ADkc=";
  meta = with lib; {
    description = "`touch`, but with templates!";
    homepage = "https://github.com/kolja/zap";
    license = licenses.mit;
    maintainers = with maintainers; [ "kolja" ];
    platforms = platforms.all;
  };
}
