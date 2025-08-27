{pkgs, ...}: {
  packages = with pkgs; [
    at-spi2-atk
    atkmm
    cairo
    gdk-pixbuf
    glib
    gtk3
    harfbuzz
    librsvg
    libsoup_3
    pango
    webkitgtk_4_1
    openssl
    xdotool

    openjdk

    bundletool

    alsa-lib
  ];

  languages.rust = {
    enable = true;
    channel = "stable";
    targets = [
      "x86_64-unknown-linux-gnu"
      "aarch64-linux-android"
      "armv7-linux-androideabi"
      "i686-linux-android"
      "x86_64-linux-android"
      "wasm32-unknown-unknown"
    ];
  };

  env.JAVA_HOME = "${pkgs.openjdk}";
  env.ANDROID_HOME = "/home/florg/Android/Sdk";
  env.ANDROID_NDK_HOME = "/home/florg/Android/Sdk/ndk/29.0.13846066";
}
