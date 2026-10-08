{
  description = "slot. - a Game Boy-centric frontend for the Anbernic RG SP";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane/v0.24.0";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      advisory-db,
      ...
    }@inputs:
    let
      inherit (nixpkgs) lib;
      forAllSystems = lib.genAttrs lib.systems.flakeExposed;
      pkgsFor =
        system:
        import nixpkgs {
          inherit system;
          overlays = [ inputs.rust-overlay.overlays.default ];
        };

      build =
        system:
        let
          pkgs = pkgsFor system;
          craneLib = (crane.mkLib pkgs).overrideToolchain (
            p: p.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml
          );
          # The crates embed their art, fonts and sound effects with
          # include_bytes!/include_str!, so the assets ride along with the Rust sources.
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              (craneLib.fileset.commonCargoSources ./.)
              ./crates/slot/assets
              ./crates/slot-ui/assets
            ];
          };
          commonArgs = {
            inherit src;
            inherit (craneLib.crateNameFromCargoToml { cargoToml = ./crates/slot/Cargo.toml; }) pname version;
            strictDeps = true;
            # The host build links cpal against ALSA and gilrs against udev on Linux.
            # The device build dlopens everything it needs, so it adds nothing here.
            nativeBuildInputs = lib.optionals pkgs.stdenv.hostPlatform.isLinux [ pkgs.pkg-config ];
            buildInputs =
              lib.optionals pkgs.stdenv.hostPlatform.isDarwin [ pkgs.libiconv ]
              ++ lib.optionals pkgs.stdenv.hostPlatform.isLinux [
                pkgs.alsa-lib
                pkgs.udev
              ];
          };
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          # The checks build in the dev profile, like `task test` and `task lint`;
          # crane's default is release.
          checkArgs = commonArgs // {
            CARGO_PROFILE = "dev";
          };
          checkArtifacts = craneLib.buildDepsOnly checkArgs;
          crate = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoExtraArgs = "--locked -p slot";
              # Tests run once, in checks.nextest.
              doCheck = false;
            }
          );
        in
        {
          inherit
            pkgs
            craneLib
            src
            commonArgs
            cargoArtifacts
            checkArgs
            checkArtifacts
            crate
            ;
        };
      builds = forAllSystems build;
    in
    {
      packages = forAllSystems (system: {
        default = builds.${system}.crate;
      });

      # The host build and its tests only target macOS: Linux runs slot on the
      # device, which builds in a container.
      checks = lib.genAttrs (lib.filter (lib.hasSuffix "-darwin") lib.systems.flakeExposed) (
        system:
        let
          b = builds.${system};
        in
        {
          build = b.crate;
          clippy = b.craneLib.cargoClippy (
            b.checkArgs
            // {
              cargoArtifacts = b.checkArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );
          # The host lint never compiles the device-only code.
          clippy-device = b.craneLib.cargoClippy (
            b.checkArgs
            // {
              cargoArtifacts = b.checkArtifacts;
              pname = "slot-device";
              cargoExtraArgs = "--locked -p slot --no-default-features --features device";
              cargoClippyExtraArgs = "-- --deny warnings";
            }
          );
          doc = b.craneLib.cargoDoc (
            b.checkArgs
            // {
              cargoArtifacts = b.checkArtifacts;
              env.RUSTDOCFLAGS = "--deny warnings";
            }
          );
          fmt = b.craneLib.cargoFmt { inherit (b.commonArgs) src pname version; };
          audit = b.craneLib.cargoAudit {
            inherit (b.commonArgs) src pname version;
            inherit advisory-db;
          };
          nextest = b.craneLib.cargoNextest (
            b.checkArgs
            // {
              cargoArtifacts = b.checkArtifacts;
              cargoNextestExtraArgs = "--no-tests=pass";
            }
          );
        }
      );

      devShells = forAllSystems (
        system:
        let
          b = builds.${system};
        in
        {
          default = b.craneLib.devShell {
            # Inherit build inputs from the checks.
            checks = self.checks.${system} or { };
            packages = with b.pkgs; [
              go-task
              cmake # task core builds mGBA
              android-tools # adb, for the deploy and log tasks
              curl
              unzip
              file
            ];
          };
        }
      );

      formatter = forAllSystems (system: (pkgsFor system).nixfmt-tree);
    };
}
