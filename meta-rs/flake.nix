{
  description = "";

  outputs = { self, nixpkgs }: {
    devShell.x86_64-linux =
      let pkgs = nixpkgs.legacyPackages.x86_64-linux;
      in pkgs.mkShell {
        buildInputs = [
          pkgs.rustc
          pkgs.cargo
          pkgs.rls
          pkgs.rustfmt
          pkgs.rustracer
          pkgs.clippy

          pkgs.gtk3
        ];

        RUST_BACKTRACE = "1";
      };
  };
}
