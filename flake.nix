{
    description = "runwhen";

    inputs = {
        flake-utils.url = "github:numtide/flake-utils";
        naersk.url = "github:nix-community/naersk";
        flake-compat = {
            url = "github:edolstra/flake-compat";
            flake = false;
        };
    };

    outputs = {self, flake-utils, naersk, flake-compat}:
    flake-utils.lib.eachDefaultSystem (system: 
    let
        naersk-lib = naersk.lib."${system}";
    in
    {
        inherit flake-compat;
        defaultPackage = naersk-lib.buildPackage rec {
                pname = "runwhen";
                version = "0.0.8";
                src = ./.;
                cargoBuildOptions = opts: opts ++ ["-p" "${pname}" ];
            };
    });
}
