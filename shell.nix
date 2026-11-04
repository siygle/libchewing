let
  pkgs = import <nixpkgs> { config = {}; overlays = []; };
in

pkgs.mkShell {
  packages = with pkgs; [
    git-lfs
    cargo
    rustc
    rustfmt
    rust-analyzer
    cmake
    ninja
    ncurses
    valgrind
    lldb
    clang-tools
  ];
}
