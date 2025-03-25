{ rustPlatform, pkg-config, openssl, lib }:
rustPlatform.buildRustPackage rec {
  pname = "musicbrainz-rss-generator";
  version = "0.1.0";

  cargoLock.lockFile = ./Cargo.lock;

  src = ./.;

  nativeBuildInputs = [ pkg-config ];

  buildInputs = [ openssl ];

  preBuild = ''
    export OPENSSL_DIR=${lib.getDev openssl}
    export OPENSSL_LIB_DIR=${lib.getLib openssl}/lib
  '';

  meta.mainProgram = pname;
}
