{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    cargo-tauri-src = {
      url = "github:tauri-apps/tauri?ref=tauri-v2.0.0-beta.22";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, cargo-tauri-src, }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # Building cargo-tauri manually since v^2 isn't available on nixpkgs
        cargo-tauri = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cargo-tauri";
          version = "2.0.0-beta.22";
          src = cargo-tauri-src;
          sourceRoot = "source/tooling/cli";
          cargoHash = "sha256-JIpQCVxK7+NMCP4rzlynA5yly1Eib9L6cIx8Q7vP7y8=";
          buildInputs = [ pkgs.openssl ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs;
            [ glibc libsoup cairo gtk3 webkitgtk ])
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks;
            [ CoreServices Security SystemConfiguration ]);
          nativeBuildInputs = [ pkgs.pkg-config ];
        };

        libraries = with pkgs;[
          webkitgtk_4_1
          gtk3
          cairo
          gdk-pixbuf
          glib
          dbus
          openssl_3
          librsvg
          libsoup
          libsoup_3
        ];

        packages = with pkgs; [
          curl
          trunk
          wget
          pkg-config
          dbus
          openssl_3
          glib
          gtk3
          libsoup
          webkitgtk_4_1
          librsvg
          rustup
          cargo-tauri
          trunk # serve web front without running a window
          hexyl # pretty hex viewer
        ];
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = packages;

          shellHook =
            ''
              export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH
              export XDG_DATA_DIRS=${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS
            '';
        };
      });
}

