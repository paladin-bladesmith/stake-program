# Based on: https://crane.dev/examples/quick-start-workspace.html

{
  inputs = {
    self.submodules = true;

    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    {
      nixpkgs,
      ...
    }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      # Dev.
      devShells.${system}.default = pkgs.mkShell {
        name = "bricklayer-shell";

        packages = [
          pkgs.nixfmt-rfc-style
        ];
        buildInputs = with pkgs; [
          openssl
          pkg-config
          rustPlatform.bindgenHook
        ];
      };
    };
}
