{
  pkgs,
  ...
}:

{
  packages = with pkgs; [
    git
    nixfmt-rfc-style

    rustup
    cargo-expand
    cargo-nextest
    cargo-workspaces

    # Required for LiteSVM tests
    openssl
    pkg-config
  ];

  env.MACOSX_DEPLOYMENT_TARGET = "13.0";
}
