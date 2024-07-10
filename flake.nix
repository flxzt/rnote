{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };
  outputs = {
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
    in {
      devShells.default = pkgs.mkShell {
        # Build inputs copied from nixpkgs https://github.com/NixOS/nixpkgs/blob/master/pkgs/applications/graphics/rnote/default.nix
        # However we are using the rust-overlay instead of using the rust provided in nixpkgs.
        buildInputs = with pkgs;
          [
            appstream
            glib
            gst_all_1.gstreamer
            gtk4
            libadwaita
            libxml2
            poppler
          ]
          ++ lib.optionals stdenv.isLinux [
            alsa-lib
          ]
          ++ lib.optionals stdenv.isDarwin [
            AudioUnit
          ];

        # Also copied from nixpkgs
        nativeBuildInputs = with pkgs.buildPackages; [
          rust-bin.stable.latest.default

          appstream-glib # For appstream-util
          desktop-file-utils # For update-desktop-database
          dos2unix
          meson
          ninja
          pkg-config
          cmake
          python3 # For the postinstall script
          rustPlatform.bindgenHook
          rustPlatform.cargoSetupHook
          shared-mime-info # For update-mime-database
          wrapGAppsHook4
        ];

        # Required on NixOS to let rnote be able to find 'Gsettings schema', otherwise the app craches on launch
        shellHook = ''
          export XDG_DATA_DIRS=$GSETTINGS_SCHEMAS_PATH:$XDG_DATA_DIRS
        '';
      };
    });
}
