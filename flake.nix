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
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [cargo2nix.overlays.default];
        };

        rustConfig = {
          rustVersion = "1.83.0";
          rootFeatures = ["day_1/no_real_inputs"];
          packageFun = import ./Cargo.nix;
          extraRustComponents = ["clippy"];
        };

        rustPkgs = pkgs.rustBuilder.makePackageSet rustConfig;
        clippyPkgs = pkgs.rustBuilder.makePackageSet ({
            packageOverrides = pkgs:
              pkgs.rustBuilder.overrides.all
              ++ [
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "all_days";
                  overrideAttrs = drv: {
                    setBuildEnv = ''
                      ${drv.setBuildEnv or ""}
                      echo
                      echo --- BUILDING WITH CLIPPY ---
                      echo
                      export RUSTC="''${CLIPPY_DRIVER}"
                    '';
                  };
                })
              ];
          }
          // rustConfig);
        day_names = builtins.attrNames (builtins.readDir self);
        days = builtins.listToAttrs (map (day_name: {
            name = day_name;
            value = rustPkgs.workspace."${day_name}" {};
          })
          day_names);
        tests = builtins.listToAttrs (map (day_name: {
            name = "test_${day_name}";
            value = pkgs.rustBuilder.runTests rustPkgs.workspace."${day_name}" {};
          })
          day_names);
        clippy_days = builtins.listToAttrs (map (day_name: {
            name = "clippy_${day_name}";
            value = clippyPkgs.workspace."${day_name}" {};
          })
          day_names);
      in {
        packages = days // clippy_days // tests;
        devShells = {
          default = pkgs.mkShell {
            packages = [
              pkgs.alejandra
              pkgs.cargo
            ];
          };
        };
      }
    );
}
