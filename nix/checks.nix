# What `nix flake check` runs besides building the packages: clippy and the unit tests of the whole
# workspace (`cargo clippy --workspace --all-targets`, `cargo test --workspace`), in the sandbox.
{
  lib,
  rustPlatform,
  clippy,
  fontconfig,
  makeFontsConf,
  dejavu_fonts,
  splice,
  splice-settings,
}:

rustPlatform.buildRustPackage {
  pname = "splice-checks";
  inherit (splice.unwrapped) version LIBCLANG_PATH;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../src
      ../crates
      ../vendor/smithay
    ];
  };

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = splice.unwrapped.nativeBuildInputs ++ splice-settings.nativeBuildInputs ++ [ clippy ];
  buildInputs = splice.unwrapped.buildInputs ++ splice-settings.buildInputs;
  # splice-ui's tests draw text: fc-match and a font it can name.
  nativeCheckInputs = [ fontconfig ];
  FONTCONFIG_FILE = makeFontsConf { fontDirectories = [ dejavu_fonts ]; };

  # The development profile, as in `nix develop`: nothing here is installed.
  buildPhase = ''
    runHook preBuild
    cargo clippy --workspace --all-targets --offline -j "$NIX_BUILD_CORES" -- -D warnings
    runHook postBuild
  '';

  checkPhase = ''
    runHook preCheck
    cargo test --workspace --offline -j "$NIX_BUILD_CORES"
    runHook postCheck
  '';

  installPhase = ''
    runHook preInstall
    touch $out
    runHook postInstall
  '';

  dontFixup = true;
}
