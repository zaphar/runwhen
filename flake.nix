{
    description = "runwhen";

    inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs";
        flake-utils.url = "github:numtide/flake-utils";
        rust-overlay = {
            url = "github:oxalica/rust-overlay";
            inputs.nixpkgs.follows = "nixpkgs";
        };
        naersk.url = "github:nix-community/naersk";
        flake-compat = {
            url = github:edolstra/flake-compat;
            flake = false;
        };
    };

    outputs = {self, nixpkgs, flake-utils, rust-overlay, naersk, flake-compat}:
    flake-utils.lib.eachDefaultSystem (system: 
    let
        overlays = [ rust-overlay.overlay ];
        pkgs = import nixpkgs { inherit system overlays; };
        naersk-lib = naersk.lib."${system}";
    in
    {
        defaultPackage = with pkgs;
            naersk-lib.buildPackage rec {
                pname = "runwhen";
                version = "0.0.4";
                src = ./.;
                cargoBuildOptions = opts: opts ++ ["-p" "${pname}" ];
            };
    });
}