{pkgs, rustPlatform}: 
rustPlatform.buildRustPackage {
  pname = "beef_market";
  src = ./.;
  version = "0.0.1";
  cargoLock.lockFile = ./Cargo.lock;
  doCheck = false;
  nativeBuildInputs = [
    pkgs.pkg-config
  ];
  buildInputs = with pkgs;[
    openssl
    openssl.dev
  ];

  postInstall = "
    cp -r migrations $out/
  ";

}
