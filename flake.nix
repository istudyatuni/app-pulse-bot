{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    ...
  }:
    utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};
      in {
        devShells.default = with pkgs;
          pkgs.mkShell {
            nativeBuildInputs = [
              just
              cargo-nextest
              sqlite
            ];
            buildInputs = [];
            CROSS_ROOTLESS_CONTAINER_ENGINE = 1;
          };
        formatter = pkgs.alejandra;
      }
    );
}
