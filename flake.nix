{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk {};
        
      in rec {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage {
          src = ./.;

          buildInputs = with pkgs; [ cmake libopus yt-dlp ffmpeg makeWrapper ];

          # Use postInstall to wrap the program with the necessary PATH
          postInstall = ''
            wrapProgram $out/bin/hoodbot \
              --prefix PATH : ${pkgs.ffmpeg}/bin:${pkgs.yt-dlp}/bin
          '';
        };

        # For `nix develop` (optional, can be skipped):
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ rustc cargo cmake pkg-config openssl ];
        };
      }
    );
}