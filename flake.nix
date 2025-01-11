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
      overlay = final: prev: {
          beef_market = self.packages.${system}.beef_market;
      };
      pkgs = nixpkgs.legacyPackages.${system}.extend overlay;
      toolchain = fenix.packages.${system}.stable.completeToolchain; 
      rustPlatform = (pkgs.makeRustPlatform {
        cargo = toolchain;
        rustc = toolchain;
      });
      
    in {
      packages.beef_market = pkgs.callPackage ./beef_market.nix { inherit rustPlatform; };

      packages.test = pkgs.callPackage ./test_beef_market.nix {inherit self;};

      checks = {
        #geckodriverTest = pkgs.callPackage ./test_geckodriver.nix {inherit self;};
        beefMarketText = pkgs.callPackage ./test_beef_market.nix {inherit self;};
      };

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
          (pkgs.python3.withPackages (python-pkgs: [
            python-pkgs.selenium
          ]))
        ];
      };
    }) // {
     nixosModules = {
        beef_market = import ./beef_market_module.nix;
        geckodriver = import ./geckodriver_module.nix;
      };
    };
}
