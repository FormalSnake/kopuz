{ lib
, rustPlatform
, pkg-config
, wrapGAppsHook3
, makeWrapper
, webkitgtk_4_1
, gtk3
, libsoup_3
, glib-networking
, alsa-lib
, xdotool
, openssl
, src
}:

let
  pname = "rusic";
  version = "0.1.1";
in
rustPlatform.buildRustPackage {
  inherit pname version src;

  cargoLock = {
    lockFile = ./../Cargo.lock;
    allowBuiltinFetchGit = true;
  };

  nativeBuildInputs = [
    pkg-config
    wrapGAppsHook3
    makeWrapper
  ];

  buildInputs = [
    webkitgtk_4_1
    gtk3
    libsoup_3
    glib-networking
    alsa-lib
    xdotool
    openssl
  ];

  cargoBuildFlags = [ "--package" "rusic" ];

  postInstall = ''
    mkdir -p $out/share/rusic/run
    mkdir -p $out/share/rusic/assets
    cp -r rusic/assets/* $out/share/rusic/assets/
    
    wrapProgram $out/bin/rusic \
      --chdir $out/share/rusic/run \
      --set GIO_MODULE_DIR "${glib-networking}/lib/gio/modules/"
    
    mkdir -p $out/share/applications
    mkdir -p $out/share/icons/hicolor/scalable/apps
    cp data/com.temidaradev.rusic.desktop $out/share/applications/
    cp rusic/assets/logo.png $out/share/icons/hicolor/scalable/apps/com.temidaradev.rusic.png
    
    substituteInPlace $out/share/applications/com.temidaradev.rusic.desktop \
      --replace "Exec=rusic" "Exec=$out/bin/rusic"
  '';

  meta = with lib; {
    description = "A modern music player built with Dioxus";
    homepage = "https://github.com/temidaradev/rusic";
    license = licenses.mit;
    maintainers = [ ];
    platforms = platforms.linux;
    mainProgram = "rusic";
  };
}
