{
  description = "Inimeg, a Gemini server built from the ground up";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs @ {
    self,
    flake-parts,
    nixpkgs,
    rust-overlay,
    flake-utils,
  }:
    flake-parts.lib.mkFlake {inherit inputs;} (
      top @ {
        config,
        lib,
        getSystem,
        ...
      }: {
        systems = nixpkgs.lib.systems.flakeExposed;
        perSystem = {
          config,
          self',
          pkgs,
          lib,
          system,
          ...
        }: let
          makeRuntimeDeps = pkgs: [pkgs.openssl];
          makeBuildDeps = pkgs: [];
          makeDevDeps = pkgs: [
          pkgs.gdb
          pkgs.pre-commit
          pkgs.cargo-nextest
          pkgs.rust-analyzer
          ];

          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          msrv = cargoToml.package.rust-version;

          mkDevShell = rustc:
            pkgs.mkShell {
              env = {
                RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
              };
              buildInputs = makeRuntimeDeps pkgs
                ;
              nativeBuildInputs = (makeBuildDeps pkgs) ++ (makeDevDeps pkgs) ++ [rustc];
              shellHook = ''
                pre-commit install
              '';
            };
          overlays = [(import rust-overlay)];

        in {
          _module.args.pkgs = import nixpkgs {inherit system overlays;};

          devShells.nightly =
            mkDevShell (pkgs.rust-bin.selectLatestNightlyWith
              (toolchain: toolchain.default));
          devShells.stable = mkDevShell pkgs.rust-bin.stable.latest.default;
          devShells.msrv = mkDevShell pkgs.rust-bin.stable.${msrv}.default;
          devShells.pinned = mkDevShell (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml);
          devShells.default = self'.devShells.pinned;
        };
      }
    );
}
