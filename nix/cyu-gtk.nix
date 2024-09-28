{
  craneLib,
}:
let
  cargoPackageConfig = builtins.fromTOML (builtins.readFile ../cyu-gtk/Cargo.toml);
  cargoWorkspaceConfig = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
craneLib.buildPackage {
  pname = cargoPackageConfig.name;
  version = cargoWorkspaceConfig.version;
  src = ../.;
  cargoLock.lockFile = ../Cargo.lock;
  cargoExtraArgs = "-p cyu-gtk";
}
