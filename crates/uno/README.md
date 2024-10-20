## How to run the examples

- You need to install the [nix package manager](https://nixos.org/) (works best on linux): https://nixos.org/download/

- You need to enable flakes for nix:
  ```
  experimental-features = nix-command flakes
  ```
  in `~/.config/nix/nix.conf`

  (See https://nixos.wiki/wiki/Flakes)

- Setup the environment:
  
  Run:
  ```
  nix develop
  ```
  to download the dependencies and setup the environment (it will take some time the first time).

- Run the programs on your arduino:
  From the `nix develop` shell, move into the `uno` directory:
  ```
  cd crates/uno
  ```
  From there run the programs with cargo, the rust build tool, e.g.:
  ```
  cargo run --bin chenillard
  ```
  
  cargo will compile + flash the arduino.

  The possible programs to run are the files in `crates/uno/src/bin`, without the `.rs` suffix.

  You may need to put your user (if not already done) in the `plugdev` group to flash the arduino.
