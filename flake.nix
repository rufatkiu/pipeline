{
  description = "Watch YouTube and PeerTube videos in one place";

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
              buildInputs = with pkgs; [ libadwaita ];
              nativeBuildInputs = with pkgs; [ wrapGAppsHook4 rustPlatform.cargoSetupHook meson gettext glib pkg-config desktop-file-utils appstream-glib ninja rustc cargo openssl ];

              inherit name;
            };
          devShells.default =
            let 
              run = pkgs.writeShellScriptBin "run" ''
                export GSETTINGS_SCHEMA_DIR=${pkgs.gtk4}/share/gsettings-schemas/${pkgs.gtk4.name}/glib-2.0/schemas/:./build/data/
                meson compile -C build && ./build/target/debug/${legacyname}
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
              nativeBuildInputs = self.packages.${system}.default.nativeBuildInputs ++ [ rustfmt python3 nodePackages.clean-css-cli libxml2 ] ++ [ run check format ];
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
