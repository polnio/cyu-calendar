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
      packages = {
        cyu-gtk = [
          {
            system = "x86_64-linux";
          }
        ];
      };

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
                ];
                buildInputs = with pkgs; [
                  gtk4
                  libadwaita
                  libsecret
                  libshumate
                ];
                PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
              };
            }
          );

      devShells = forAllSystem (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              rustc
              cargo

              # CYU GTK
              pkg-config
              gtk4
              libadwaita
              libsecret
              libshumate
            ];
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          };
        }
      );
    };
}
