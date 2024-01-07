{
  description = "The server-side software for Remote Text";

  inputs = {
    # nixpkgs.url = "github:NixOS/nixpkgs";
    flockenzeit.url = "github:balsoft/Flockenzeit";
  };

  outputs = { self, flockenzeit, nixpkgs, ... }:
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
          created = flockenzeit.lib.ISO-8601 self.lastModified;
          config = {
            Cmd = [ "${remote-text-server}/bin/remote-text-server" ];
          };
        };
      });
    };
}
