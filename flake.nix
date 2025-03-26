{
  description = "System service for monitoring and optimizing app data usage";

  inputs = {
    nixpkgs.url =
      "github:NixOS/nixpkgs?rev=a47b881e04af1dd6d414618846407b2d6c759380";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        # manifest = pkgs.lib.importTOML ./Cargo.toml;
        # package = manifest.package;
        # data-guardian = pkgs.rustPlatform.buildRustPackage {
        #   pname = package.name;
        #   version = package.version;
        #   src = pkgs.lib.cleanSource ./.;
        #   cargoLock.lockFile = ./Cargo.lock;
        #   meta = with pkgs.lib; {
        #     inherit (package) description homepage repository;
        #     license = licenses.mit;
        #     maintainers = [ maintainers.xosnrdev ];
        #   };
        #   doCheck = false;
        # };

        devShell = pkgs.mkShell {
          buildInputs = [
            pkgs.cargo-watch
            pkgs.cargo-sort
            pkgs.cargo-release
            pkgs.cargo-edit
            pkgs.cargo-dist
            pkgs.cargo-audit
            pkgs.git
          ];
          shellHook = ''
            export RUST_BACKTRACE=1
            export RUST_LOG=debug
          '';
        };

      in {
        formatter = pkgs.nixfmt-classic;
        # packages = { default = data-guardian; };
        devShells.default = devShell;
      });
}
