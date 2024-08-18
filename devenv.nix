{ pkgs, lib, ... }:

{
  # https://devenv.sh/packages/
  packages =
    with pkgs;
    [
      git
      jq
      openssl
      tree
    ]
    ++ lib.optionals pkgs.stdenv.isDarwin [
      libiconv
      darwin.apple_sdk.frameworks.SystemConfiguration
    ];

  # https://devenv.sh/languages/
  languages.rust.enable = true;

  enterShell = ''
    echo "In arkive development shell"
  '';

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests"
    cargo test
  '';

  # https://devenv.sh/pre-commit-hooks/
  pre-commit.hooks.rustfmt.enable = true;
  pre-commit.hooks.deadnix.enable = true; # dead nix code remover
  pre-commit.hooks.nixfmt.enable = true;
  pre-commit.hooks.nixfmt.package = pkgs.nixfmt-rfc-style;
  pre-commit.hooks.check-merge-conflicts.enable = true;
  pre-commit.hooks.check-added-large-files.enable = true; # prohibit "very large" files
  pre-commit.hooks.check-case-conflicts.enable = true; # macos fs is case-insensitive; helps linux
  pre-commit.hooks.check-json.enable = true;
  pre-commit.hooks.check-toml.enable = true;
  pre-commit.hooks.check-yaml.enable = true;
  pre-commit.hooks.trim-trailing-whitespace.enable = true;
  pre-commit.hooks.shellcheck.enable = true;

  # See full reference at https://devenv.sh/reference/options/
}
