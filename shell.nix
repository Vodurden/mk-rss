let
  sources = import nix/sources.nix;
  pkgs = import sources.nixpkgs { overlays = [(import sources.nixpkgs-mozilla)]; };
  unstable = import sources.nixpkgs-unstable {};

  # Anything higher then 1.45 seems to cause the musl binaries to still be linked
  # to glibc which triggers a segfault.
  rustChannel = pkgs.rustChannelOfTargets "1.45" null [
    "x86_64-unknown-linux-musl"
  ];
in

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    # The rust-analyzer from `rustChannel` is slightly broken, so we use the nixpkgs one instead
    #
    # It's crucial that rust-analyzer appears before rust, because the order of buildInputs determines the PATH environment variable.
    #
    # See: https://github.com/mozilla/nixpkgs-mozilla/issues/238#issuecomment-714577687
    rust-analyzer

    rustChannel

    rustfmt

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
