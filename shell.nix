{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    clippy
    rustfmt
    git
  ];

  shellHook = ''
    echo "skillz development environment"
    echo "rust: $(rustc --version)"
    echo ""
    echo "commands:"
    echo "  cargo build       - build debug"
    echo "  cargo build --release - build release"
    echo "  cargo test        - run tests"
    echo "  cargo clippy      - lint"
    echo "  cargo fmt         - format code"
  '';
}
