{
  description = "Rust development template";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self
    , nixpkgs
    , utils
    , rust-overlay
    , crane
    , ...
    }:
    utils.lib.eachDefaultSystem
      (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              rust-overlay.overlays.default
            ];
          };

          inherit (pkgs) lib;

          craneLib = crane.mkLib pkgs;
          # include proto & compiled json 
          protoFilter = path: _type: builtins.match ".*proto$" path != null;
          jsonFilter = path: _type: builtins.match ".*json$" path != null;
          sourceFilter = path: type:
            (protoFilter path type) || (jsonFilter path type) || (craneLib.filterCargoSources path type);


          src = lib.cleanSourceWith {
            src = ./.;
            filter = sourceFilter;
            name = "source"; # Be reproducible, regardless of the directory name
          };

          loadedChain = pkgs.rust-bin.stable."1.80.0".default.override {
            extensions = [ "rust-src" ];
            targets = [ "wasm32-unknown-unknown" ];
          };

          craneLibLLvmTools =
            craneLib.overrideToolchain
              loadedChain;

          cairo-zip = pkgs.fetchurl {
            url = "https://github.com/starkware-libs/cairo/archive/refs/tags/v2.7.0.zip";
            hash = "sha256-jjLEHBXsfCu2CSoXvpev0HMzHxoc2rYE9PsVonPVuTI=";
          };

          commonArgs = {
            inherit src;

            pname = "dojo";
            strictDeps = true;
            doCheck = false;

            buildInputs = with pkgs;
              [
                curl
                openssl
                libclang
                libclang.lib

                protobuf
              ]
              ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
                # Additional darwin specific inputs can be set here
                pkgs.libiconv
              ];

            nativeBuildInputs = with pkgs; [
              rustPlatform.bindgenHook
              pkg-config
            ];

            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            CAIRO_ARCHIVE = "${cairo-zip}";
            PROTOC = "${pkgs.protobuf}/bin/protoc";
          };


          # Override for scarb-metadata (pin versions for now)
          isScarb = p: lib.hasPrefix
            "git+https://github.com/software-mansion/scarb"
            p.source;

          cargoVendorDir = craneLib.vendorCargoDeps (commonArgs // {
            # Use this function to override crates coming from git dependencies
            overrideVendorGitCheckout = ps: drv:
              # For example, patch a specific repository and tag, in this case num_cpus-1.13.1
              if lib.any (p: (isScarb p)) ps then
                drv.overrideAttrs
                  (_old:
                    let
                      pss = lib.findFirst (p: (p.name == "scarb-build-metadata")) null ps;
                      scarb = lib.findFirst (p: (p.name == "scarb")) null ps;
                    in
                    {

                      patches = [
                        (pkgs.substituteAll {
                          src = ./.nix-hack/scarb-metadata.patch;
                          cairoZip = "${cairo-zip}";
                        })
                      ];

                      # Similarly we can also run additional hooks to make changes
                      postInstall = builtins.trace pss ''
                        echo "=========="
                        echo "-> " $CAIRO_ARCHIVE
                        SCARB_META_OUT_DIR=${pss.name}-${pss.version}
                        cp $src/Cargo.lock $out/$SCARB_META_OUT_DIR/Cargo.lock
                        echo --- Fix values
                        CAIRO_VERSION=$(${pkgs.toml-cli}/bin/toml get Cargo.lock . | jq '.package[] | select(.name == "cairo-lang-compiler").version' -r)
                        sed -i -e "s/{{cairo_version}}/$CAIRO_VERSION/g" $out/$SCARB_META_OUT_DIR/build.rs
                        sed -i -e "s/{{version}}/${scarb.version}/g" $out/$SCARB_META_OUT_DIR/build.rs
                        echo "=========="
                        cat $out/$SCARB_META_OUT_DIR/build.rs
                      '';
                    }
                  )
              else
              # Nothing to change, leave the derivations as is
                drv;
          });

          cargoArtifactsValue = commonArgs // {
            cargoVendorDir = builtins.toString cargoVendorDir;
          };

          cargoArtifacts = craneLib.buildDepsOnly cargoArtifactsValue;

          individualCrateArgs =
            commonArgs
            // {
              inherit cargoArtifacts cargoVendorDir;
              inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
              # NB: we disable tests since we'll run them all via cargo-nextest
              doCheck = false;
            };

          crates = builtins.map (x: ./. + "/crates/${x}") (builtins.attrNames (builtins.readDir ./crates));
          fileSetForCrate = crate:
            pkgs.lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions ([
                ./Cargo.toml
                ./Cargo.lock
                crate
              ] ++ crates);
            };
        in
        rec
        {
          packages = {
          };

          # Used by `nix develop`
          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              loadedChain

              clippy
              rustfmt

              pkg-config

              openssl
              libclang
              libclang.lib

              protobuf
            ];

            nativeBuildInputs = with pkgs; [
              rustPlatform.bindgenHook
              pkg-config
            ];

            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            CAIRO_ARCHIVE = "${cairo-zip}";
            PROTOC = "${pkgs.protobuf}/bin/protoc";
          };
        }
      );
}

