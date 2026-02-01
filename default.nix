{
  rustPlatform,
  features ? "",
  cargoToml,
  makeRuntimeDeps,
  makeBuildDeps,
  pkgs,
}:
rustPlatform.buildRustPackage {
  inherit (cargoToml.package) name version;
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;
  cargoLock.allowBuiltinFetchGit = true;
  buildFeatures = features;
  buildInputs = makeRuntimeDeps pkgs;
  nativeBuildInputs = makeBuildDeps pkgs;
  # Uncomment if your cargo tests require networking or otherwise
  # don't play nicely with the Nix build sandbox:
  # doCheck = false;
}
