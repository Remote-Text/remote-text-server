{
  description = "The server-side software for Remote Text";

  outputs = { nixpkgs, ... }:
    let
      forAllSystems = gen:
        nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed
        (system: gen nixpkgs.legacyPackages.${system});
    in {
      packages = forAllSystems (pkgs: rec {
        remote-text-server = pkgs.callPackage ./. { };
        default = remote-text-server;
        dockerImage = pkgs.dockerTools.buildImage {
          name = "remote-text-server";
          config = {
            Cmd = [ "${remote-text-server}/bin/remote-text-server" ];
          };
        };
      });
    };
}
