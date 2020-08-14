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

    # library dependencies
    pkgconfig
  ];

  PKG_CONFIG_ALLOW_CROSS=true;
  PKG_CONFIG_ALL_STATIC=true;
  LIBZ_SYS_STATIC=1;

  OPENSSL_STATIC=1;
  OPENSSL_DIR = pkgs.pkgsStatic.openssl.dev;
  OPENSSL_LIB_DIR = "${pkgs.pkgsStatic.openssl.out}/lib";
}
