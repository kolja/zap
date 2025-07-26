# flake.nix
    {
      description = "`touch`, but with templates!";

      inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
        flake-utils.url = "github:numtide/flake-utils";
      };

      outputs = { self, nixpkgs, flake-utils }:
        flake-utils.lib.eachDefaultSystem (system:
          let
            pkgs = import nixpkgs {
              inherit system;
            };
          in
          {
            packages = {
              zap = pkgs.callPackage ./default.nix { };
              default = self.packages.${system}.zap;
            };

            apps.default = {
              type = "app";
              program = "${self.packages.${system}.default}/bin/zap";
            };

            devShells.default = pkgs.mkShell {
                packages = with pkgs; [
                    (rust-bin.stable.latest.default.override {
                        extensions = [ "rust-src" "rust-analyzer" ];
                    })
                    cargo-watch
                ];
            };
          });
    }
