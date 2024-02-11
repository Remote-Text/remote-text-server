{ lib
, rustPlatform
, pkg-config
, libgit2
, openssl
, zlib
, stdenv
, darwin
, pandoc
, texlive
, makeWrapper
}:

rustPlatform.buildRustPackage rec {
  pname = "remote-text-server";
  version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;

  src = ./.;

  cargoHash = "sha256-g6QiGH9eqC/mrGzeZOJ5wqm5V5D2xsDm4OOyzmE4sqM=";

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    libgit2
    openssl
    zlib
    makeWrapper
  ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.IOKit
    darwin.apple_sdk.frameworks.Security
  ];

  postFixup = ''
    wrapProgram $out/bin/remote-text-server \
      --set PATH ${lib.makeBinPath [
        pandoc
        texlive
      ]}
  '';

  env = {
    OPENSSL_NO_VENDOR = true;
    VERGEN_IDEMPOTENT = true;
  };

  meta = with lib; {
    description = "The server-side software for Remote Text";
    homepage = "https://github.com/Remote-Text/remote-text-server";
    license = with licenses; [ ];
    maintainers = with maintainers; [ ];
  };
}
