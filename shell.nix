with import <nixpkgs> {};
let
    shared = [
        rustup
        shellcheck
    ];
    hook = ''
        . .shellhook
    '';
in
{
    darwin = mkShell {
        buildInputs = shared;
        shellHook = hook;
    };
    linux = mkShell {
        buildInputs = [
            pkg-config
        ] ++ shared;
        APPEND_LIBRARY_PATH = stdenv.lib.makeLibraryPath [
            libGL
            linuxPackages.nvidia_x11
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
        ];
        shellHook = ''
            export LD_LIBRARY_PATH="$APPEND_LIBRARY_PATH:$LD_LIBRARY_PATH"
            expression=$(grep "export" < nixGL/result/bin/nixGLNvidia)
            if [ -n "$expression" ]; then
                eval "$expression"
            fi
        '' + hook;
    };
}
