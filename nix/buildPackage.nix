pkgs: args:
let
  cargoWorkspaceConfig = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
pkgs.rustPlatform.buildRustPackage (
  args
  // {
    version = cargoWorkspaceConfig.workspace.package.version;
    src = ../.;
    cargoLock.lockFile = ../Cargo.lock;
    cargoBuildFlags = [
      "-p"
      "${args.pname}"
    ];
  }
)
