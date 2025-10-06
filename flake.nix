{
  inputs.nixpkgs.url = "github:nixos/nixpkgs";
  inputs.fenix = {
    url = "github:nix-community/fenix";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {nixpkgs, fenix, ...}: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;};

    rustToolchain = fenix.packages.${system}.fromToolchainFile {
      file = ./rust-toolchain.toml;
      sha256 = "sha256-lA+P/gO+FPBym55ZoYV9nZiIxCEXAW4tYUi5OQnj/10=";
    };
  in {
    devShells.${system}.default = pkgs.mkShell {
      packages = [rustToolchain];
    };
  };
}
