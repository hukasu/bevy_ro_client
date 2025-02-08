let
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/tarball/nixos-24.11";
  rust-overlay = (import (builtins.fetchGit {
    url = "https://github.com/oxalica/rust-overlay";
    ref = "master";
  }));
  pkgs = import nixpkgs { config = {}; overlays = [ rust-overlay ]; };
  system = builtins.currentSystem;
  extensions =
    (import (builtins.fetchGit {
      url = "https://github.com/nix-community/nix-vscode-extensions";
      ref = "master";
      rev = "a6df283f4762b079b4d09b25acb1d9bd95f6a472";
    })).extensions.${system};
  extensionsList = with extensions.vscode-marketplace; [
      rust-lang.rust-analyzer
      tamasfe.even-better-toml
      usernamehw.errorlens
      serayuzgur.crates
      vadimcn.vscode-lldb
      notblank00.hexeditor
  ];
  buildInputs = with pkgs; [
    udev
    alsa-lib
    libglvnd
    vulkan-loader
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    libxkbcommon
    wayland
    lldb
  ];
in
  pkgs.mkShell {
    nativeBuildInputs = with pkgs; [
      pkg-config
    ];
    inherit buildInputs;
    packages = with pkgs; [
      git
      tracy
      (rust-bin.stable.latest.default.override { extensions = ["rust-src"]; })
      (vscode-with-extensions.override {
        vscode = vscodium;
        vscodeExtensions = extensionsList;
      })
    ];
    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
    LLDB_DEBUGSERVER_PATH = "${pkgs.lldb}/bin/lldb-server";
    NIXOS_OZONE_WL=1;
  }
