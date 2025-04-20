{
  description = "staticify-ip - A tool for reconfiguring cloudflare servers";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    cargo2nix = {
      url = "github:cargo2nix/cargo2nix/release-0.11.0";
      inputs.rust-overlay.follows = "rust-overlay";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, cargo2nix } @ inputs:
    let
      overlays = [ (import rust-overlay) cargo2nix.overlays.default];
    in
    {
      packages = flake-utils.lib.eachDefaultSystemMap
        (system: let
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          rustPkgs = pkgs.rustBuilder.makePackageSet {
            rustVersion = rustToolchain.version;
            packageFun = import ./Cargo.nix;
          };
        in {
            staticify-ip = (rustPkgs.workspace.staticify-ip {});
            default = inputs.self.packages.${system}.staticify-ip;
        }
      );

      nixosModules.default = import ./modules/nixos.nix inputs;

      devShells = flake-utils.lib.eachDefaultSystemMap
        (system: let
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        in
        with pkgs; {
          default = mkShell {
            buildInputs = [
              rustToolchain
              openssl
              pkg-config
              cocogitto
              cargo2nix.packages.${system}.cargo2nix
            ];
          };
        }
      );
    };
}
