{
  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/*.tar.gz";
    rust-overlay.url = "https://flakehub.com/f/oxalica/rust-overlay/*.tar.gz";
    flake-utils.url = "https://flakehub.com/f/numtide/flake-utils/0.1.92.tar.gz";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ...  }: 
  flake-utils.lib.eachDefaultSystem(system:
    let 
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };
    in 
    {
      devShells.default = with pkgs; mkShell {
        buildInputs = [
          (rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" "rustfmt" ];
            targets = [ "wasm32-wasip1" ];
          })

          wasm-tools

          cargo
        ];
      };
    });
}