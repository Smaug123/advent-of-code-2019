{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.11.0";
    cargo2nix.inputs.rust-overlay.follows = "rust-overlay";
    flake-utils.follows = "cargo2nix/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.follows = "rust-overlay/nixpkgs";
  };

  outputs = {
    flake-utils,
    self,
    nixpkgs,
    cargo2nix,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            cargo2nix.overlays.default
            (import rust-overlay)
          ];
        };

        crates = builtins.attrNames (builtins.readDir self);

        rustConfig = {
          rustVersion = "1.83.0";
          rootFeatures = map (x: "${x}/no_real_inputs") crates;
          packageFun = import ./Cargo.nix;
          extraRustComponents = ["clippy"];
        };

        rustPkgs = pkgs.rustBuilder.makePackageSet rustConfig;
        clippyPkgs = pkgs.rustBuilder.makePackageSet ({
            packageOverrides = pkgs:
              pkgs.rustBuilder.overrides.all
              ++ map (
                crate: (pkgs.rustBuilder.rustLib.makeOverride {
                  name = crate;
                  overrideAttrs = drv: {
                    setBuildEnv = ''
                      ${drv.setBuildEnv or ""}
                      echo
                      echo --- BUILDING WITH CLIPPY "''${CLIPPY_DRIVER}" ---
                      echo
                      export RUSTC="''${CLIPPY_DRIVER}"
                      export RUSTFLAGS="-Dwarnings"
                    '';
                  };
                })
              )
              crates;
          }
          // rustConfig);
        days = builtins.listToAttrs (map (crates: {
            name = crates;
            value = rustPkgs.workspace."${crates}" {};
          })
          crates);
        tests = builtins.listToAttrs (map (crates: {
            name = "test_${crates}";
            value = pkgs.rustBuilder.runTests rustPkgs.workspace."${crates}" {
              testCommand = bin: ''
                INSTA_UPDATE=no INSTA_WORKSPACE_ROOT="${self}" "${bin}"
              '';
            };
          })
          crates);
        clippy_days = builtins.listToAttrs (map (crates: {
            name = "clippy_${crates}";
            value = clippyPkgs.workspace."${crates}" {};
          })
          crates);
      in {
        packages = days // clippy_days // tests;
        devShells = {
          default = pkgs.mkShell {
            packages = [
              pkgs.alejandra
              (pkgs.rust-bin.stable."${rustConfig.rustVersion}".default.override {
                extensions = ["rust-src" "clippy" "rustfmt"];
              })
              pkgs.cargo-insta
            ];
          };
        };
      }
    );
}
