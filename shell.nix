let
  sources = import nix/sources.nix;
  pkgs = import sources.nixpkgs { overlays = [(import sources.nixpkgs-mozilla)]; };
  unstable = import sources.nixpkgs-unstable {};

  rustChannel = pkgs.rustChannelOfTargets "stable" null [
    "x86_64-unknown-linux-musl"
  ];
in

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    rustChannel

    rustfmt
    unstable.rust-analyzer

    zip
    terraform
  ];
}
