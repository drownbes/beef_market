{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { self, fenix, flake-utils, nixpkgs }: 
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      toolchain = fenix.packages.${system}.stable.completeToolchain; 
      rustPlatform = (pkgs.makeRustPlatform {
        cargo = toolchain;
        rustc = toolchain;
      });
    in {
      
      nixosModules.beef_market = import ./beef_market_module.nix;
      packages.beef_market = pkgs.callPackage ./beef_market.nix { inherit rustPlatform; };

      devShells.default = pkgs.mkShell {
        shellHook = ''
          export SHELL="${pkgs.bashInteractive}/bin/bash"
          export DATABASE_URL=sqlite://db.db
          source "${toolchain}/etc/bash_completion.d/cargo"
        '';
        nativeBuildInputs = [
            pkgs.pkg-config
        ];
        buildInputs = with pkgs;[
          toolchain
          just
          openssl
          openssl.dev
          geckodriver
          sqlx-cli
          sqlite
          sqlite-vec
          litecli
        ];
      };
    });
}
