pkgs: args:
let
  cargoWorkspaceConfig = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
pkgs.rustPlatform.buildRustPackage (
  args
  // {
    version = cargoWorkspaceConfig.workspace.package.version;
    src = ../.;
    cargoLock = {
      lockFile = ../Cargo.lock;
      outputHashes = {
        "auth-token-0.1.0" = "sha256-Tk1GtYEYlmMyFEB3oqADBb1Q7N8T1P6SyaeI2MFukSM=";
      };
    };
    cargoBuildFlags = [
      "-p"
      "${args.pname}"
    ];
    cargoTestFlags = [
      "-p"
      "${args.pname}"
    ];
  }
)
