{
  description = "Terminal Markdown Viewer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
          lib = pkgs.lib;
          mdv = pkgs.rustPlatform.buildRustPackage {
            pname = "mdv";
            version = cargoToml.package.version;
            src = lib.cleanSourceWith {
              src = ./.;
              filter = path: type:
                let
                  name = baseNameOf path;
                in
                !(builtins.elem name [
                  ".git"
                  ".tmp"
                  "dist"
                  "target"
                ]);
            };
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [
              pkgs.pkg-config
            ];
            buildInputs = [
              pkgs.oniguruma
            ];
            doCheck = false;
            meta = {
              description = cargoToml.package.description;
              homepage = cargoToml.package.repository;
              license = lib.licenses.mit;
              mainProgram = "mdv";
              platforms = systems;
            };
          };
        in
        {
          default = mdv;
          mdv = mdv;
        });

      apps = forAllSystems (system:
        let
          package = self.packages.${system}.default;
        in
        {
          default = {
            type = "app";
            program = "${package}/bin/mdv";
          };
        });

      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.default ];
            packages = [
              pkgs.cargo
              pkgs.clippy
              pkgs.rustc
              pkgs.rustfmt
            ];
          };
        });
    };
}
