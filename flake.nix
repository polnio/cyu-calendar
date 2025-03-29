{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
    }:
    let
      lib = nixpkgs.lib;
      forAllSystem = lib.genAttrs [ "x86_64-linux" ];
      buildPackage = import ./nix/buildPackage.nix;
    in
    {
      packages =
        nixpkgs.lib.genAttrs
          [
            "x86_64-linux"
            "aarch64-linux"
          ]
          (
            system:
            let
              pkgs = import nixpkgs { inherit system; };
            in
            {
              cyu-gtk = buildPackage pkgs {
                pname = "cyu-gtk";
                nativeBuildInputs = with pkgs; [
                  pkg-config
                  wrapGAppsHook
                  rustPlatform.bindgenHook
                ];
                buildInputs = with pkgs; [
                  gtk4
                  libadwaita
                  libsecret
                  libshumate
                ];
                PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
              };
              cyu-api = buildPackage pkgs rec {
                pname = "cyu-api";
                nativeBuildInputs = with pkgs; [
                  nodejs
                  pkg-config
                  rustPlatform.bindgenHook
                  importNpmLock.npmConfigHook
                ];
                buildInputs = with pkgs; [
                  sqlite
                  pango
                ];
                npmRoot = pname;
                npmDeps = pkgs.importNpmLock {
                  npmRoot = ./${pname};
                };
                preBuild = ''
                  cd ${pname}
                  npm run build
                  cd ..
                '';
                PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
                SQLX_OFFLINE = "true";
              };
            }
          );

      devShells = forAllSystem (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell rec {
            nativeBuildInputs = with pkgs; [
              rustc
              cargo
              nodejs
              sqlx-cli

              # CYU GTK
              pkg-config
              gtk4
              libadwaita
              libsecret
              libshumate

              # CYU API
              sqlite
              clang
              libclang
            ];
            LD_LIBRARY_PATH = lib.makeLibraryPath nativeBuildInputs;
            LiBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
          };
        }
      );
    };
}
