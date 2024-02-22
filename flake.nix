{
  description = "Follow video creators";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils, ... }@inputs:
    (flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
          name = "pipeline";
          legacyname = "tubefeeder";
          appid = "de.schmidhuberj.tubefeeder";
        in
        rec { 
          packages.default = 
            with pkgs;
            stdenv.mkDerivation rec {
              cargoDeps = rustPlatform.importCargoLock {
                lockFile = ./Cargo.lock;

                outputHashes = {
                  "tf_core-0.1.4" = "sha256-yOuvHLyX/qUJSs62VbripKwIEoErsPu9rzbKMdndvmc=";
                  "tf_join-0.1.7" = "sha256-yOuvHLyX/qUJSs62VbripKwIEoErsPu9rzbKMdndvmc=";
                  "tf_filter-0.1.3" = "sha256-yOuvHLyX/qUJSs62VbripKwIEoErsPu9rzbKMdndvmc=";
                  "tf_observer-0.1.3" = "sha256-yOuvHLyX/qUJSs62VbripKwIEoErsPu9rzbKMdndvmc=";
                  "tf_playlist-0.1.4" = "sha256-yOuvHLyX/qUJSs62VbripKwIEoErsPu9rzbKMdndvmc=";
                  "tf_platform_youtube-0.1.7" = "sha256-yOuvHLyX/qUJSs62VbripKwIEoErsPu9rzbKMdndvmc=";
                  "tf_platform_peertube-0.1.5" = "sha256-yOuvHLyX/qUJSs62VbripKwIEoErsPu9rzbKMdndvmc=";
                };
              };
              src = ./.;
              buildInputs = with pkgs; [ pkgs.libadwaita ];
              nativeBuildInputs = with pkgs; [ pkgs.wrapGAppsHook4 rustPlatform.cargoSetupHook meson gettext pkgs.glib pkg-config pkgs.desktop-file-utils pkgs.appstream-glib ninja rustc cargo openssl ];

              inherit name;
            };
          devShells.default =
            let 
              run = pkgs.writeShellScriptBin "run" ''
                export GSETTINGS_SCHEMA_DIR=${pkgs.gtk4}/share/gsettings-schemas/${pkgs.gtk4.name}/glib-2.0/schemas/:${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}/glib-2.0/schemas/:./build/data/
                meson compile -C build && ./build/target/debug/${legacyname}
              '';
              debug = pkgs.writeShellScriptBin "debug" ''
                export GSETTINGS_SCHEMA_DIR=${pkgs.gtk4}/share/gsettings-schemas/${pkgs.gtk4.name}/glib-2.0/schemas/:${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}/glib-2.0/schemas/:./build/data/
                meson compile -C build && gdb ./build/target/debug/${legacyname}
              '';
              check = pkgs.writeShellScriptBin "check" ''
                cargo clippy
              '';
              format = pkgs.writeShellScriptBin "format" ''
                cargo fmt
                python3 -m json.tool build-aux/${appid}.json build-aux/${appid}.json
                xmllint --format --recover data/resources/resources.gresource.xml -o data/resources/resources.gresource.xml
                xmllint --format --recover data/${appid}.gschema.xml -o data/${appid}.gschema.xml
                xmllint --format --recover data/${appid}.metainfo.xml -o data/${appid}.metainfo.xml
                cleancss --format beautify -o data/resources/style.css data/resources/style.css
              '';
            in
            with pkgs;
            pkgs.mkShell {
              src = ./.;
              buildInputs = self.packages.${system}.default.buildInputs;
              nativeBuildInputs = self.packages.${system}.default.nativeBuildInputs ++ [ rustfmt python3 nodePackages.clean-css-cli libxml2 gdb cargo-deny ] ++ [ run check format debug ];
              shellHook = ''
                meson setup -Dprofile=development build
              '';
            };
          apps.default = {
            type = "app";
            inherit name;
            program = "${self.packages.${system}.default}/bin/${name}";
          };
        })
    );
}
