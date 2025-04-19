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

  outputs = { self, nixpkgs, flake-utils, rust-overlay, cargo2nix }: 
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) cargo2nix.overlays.default];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          rustPkgs = pkgs.rustBuilder.makePackageSet {
            rustVersion = rustToolchain.version;
            packageFun = import ./Cargo.nix;
          };
        in
        with pkgs;
        rec
        {
          packages = {
            staticify-ip = (rustPkgs.workspace.staticify-ip {});
            default = packages.staticify-ip;
         };
          devShells.default = mkShell {
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
}
