{
  description = "Watch YouTube and PeerTube videos in one place";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.nixpkgsgnome.url = "github:NixOS/nixpkgs/704a204ff98d07aec17bd741d304c0ea8c30bc49";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, nixpkgsgnome, flake-utils, ... }@inputs:
    (flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
          pkgsgnome = import nixpkgsgnome {
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
                  "tf_core-0.1.4" = "sha256-nAqoI0/LaHDmdhm/3Z2WLkOMbjkvLU6VD2haEi/YFJc=";
                  "tf_join-0.1.7" = "sha256-nAqoI0/LaHDmdhm/3Z2WLkOMbjkvLU6VD2haEi/YFJc=";
                  "tf_filter-0.1.3" = "sha256-nAqoI0/LaHDmdhm/3Z2WLkOMbjkvLU6VD2haEi/YFJc=";
                  "tf_observer-0.1.3" = "sha256-nAqoI0/LaHDmdhm/3Z2WLkOMbjkvLU6VD2haEi/YFJc=";
                  "tf_playlist-0.1.4" = "sha256-nAqoI0/LaHDmdhm/3Z2WLkOMbjkvLU6VD2haEi/YFJc=";
                  "tf_platform_youtube-0.1.7" = "sha256-nAqoI0/LaHDmdhm/3Z2WLkOMbjkvLU6VD2haEi/YFJc=";
                  "tf_platform_peertube-0.1.5" = "sha256-nAqoI0/LaHDmdhm/3Z2WLkOMbjkvLU6VD2haEi/YFJc=";
                };
              };
              src = ./.;
              buildInputs = with pkgs; [ pkgsgnome.libadwaita ];
              nativeBuildInputs = with pkgs; [ pkgsgnome.wrapGAppsHook4 rustPlatform.cargoSetupHook meson gettext pkgsgnome.glib pkg-config pkgsgnome.desktop-file-utils pkgsgnome.appstream-glib ninja rustc cargo openssl ];

              inherit name;
            };
          devShells.default =
            let 
              run = pkgs.writeShellScriptBin "run" ''
                export GSETTINGS_SCHEMA_DIR=${pkgs.gtk4}/share/gsettings-schemas/${pkgs.gtk4.name}/glib-2.0/schemas/:${pkgsgnome.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgsgnome.gsettings-desktop-schemas.name}/glib-2.0/schemas/:./build/data/
                meson compile -C build && ./build/target/debug/${legacyname}
              '';
              debug = pkgs.writeShellScriptBin "debug" ''
                export GSETTINGS_SCHEMA_DIR=${pkgs.gtk4}/share/gsettings-schemas/${pkgs.gtk4.name}/glib-2.0/schemas/:${pkgsgnome.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgsgnome.gsettings-desktop-schemas.name}/glib-2.0/schemas/:./build/data/
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
              nativeBuildInputs = self.packages.${system}.default.nativeBuildInputs ++ [ rustfmt python3 nodePackages.clean-css-cli libxml2 gdb ] ++ [ run check format debug ];
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
