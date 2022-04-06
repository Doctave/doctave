{
  description = "an opinionated documentation site generator";

  # inputs needed for rust along with some nice tooling for development environment
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, utils, rust-overlay, naersk, ... }:
    let
      name = "doctave";
    in
    utils.lib.eachDefaultSystem (system:
      let
        # nix packages with rust overlay defaulats instead
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlay
            (self: super: {
              rustc = self.rust-bin.stable.latest.default;
              cargo = self.rust-bin.stable.latest.default;
            })
          ];
        };

        # override version cargo and rustc version of naersk with overlay
        naersk-lib = naersk.lib.${system}.override {
          cargo = pkgs.cargo;
          rustc = pkgs.rustc;
        };

      in
      rec {
        # main package to base all builds from
        packages.${name} = naersk-lib.buildPackage {
          pname = name;
          # root of rust source holding cargo.lock
          root = ./.;
        };
        defaultPackage = packages.${name};
        # build out a container image if want to run as a container
        packages.container = pkgs.dockerTools.buildImage {
          inherit name;
          tag = packages.${name}.version;
          created = "now";
          contents = packages.${name};
          config.Cmd = [ "${packages.${name}}/bin/doctave" ];
        };
        # useful default app to run from nix directly
        apps.${name} = utils.lib.mkApp {
          inherit name;
          drv = packages.${name};
        };
        defaultApp = apps.${name};

        # dev shell default
        devShell = pkgs.mkShell {
          shellHook = ''
            export RUST_SRC_PATH=${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}
          '';

          nativeBuildInputs = [ pkgs.cargo ];
        };
      }
    );
}
