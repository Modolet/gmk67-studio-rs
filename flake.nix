# @file flake.nix
# @brief Rust / GPUI / HID development environment
# @author modolet <y@xxyx.io>
# @date 2026-10-12
{
  description = "GMK67 Studio Rust development environment";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAll = nixpkgs.lib.genAttrs systems;
    in
    {
      devShells = forAll (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          libraries = with pkgs; [
            alsa-lib
            fontconfig
            systemd.dev
            wayland
            libxkbcommon
            libx11
            libxcursor
            libxi
            libxrandr
            libxcb
            libxext
            vulkan-loader
            mesa
            libglvnd
            openssl
            zstd
          ];
        in
        {
          default = pkgs.mkShell {
            packages =
              (with pkgs; [
                rustc
                cargo
                rustfmt
                clippy
                pkg-config
                clang
                mold
                sccache
              ])
              ++ libraries;
            RUSTC_WRAPPER = "sccache";
            CARGO_INCREMENTAL = "0";
            SCCACHE_CACHE_SIZE = "100G";
            shellHook = ''
              export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath libraries}:''${LD_LIBRARY_PATH:-}"
            '';
          };
        }
      );
    };
}
