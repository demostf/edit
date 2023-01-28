{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/release-22.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    rust-overlay,
  }:
    utils.lib.eachDefaultSystem (system: let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
    in rec {
      devShell = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          (rust-bin.stable.latest.default.override {
            targets = [ "wasm32-unknown-unknown" ];
          })
          rustc
          cargo
          bacon
          cargo-edit
          cargo-outdated
          clippy
          cargo-audit
          hyperfine
          valgrind
          wasm-pack
          pkg-config
          nodejs
        ];

        buildInputs = with pkgs; [openssl];
        OPENSSL_NO_VENDOR = 1;
      };
    });
}
