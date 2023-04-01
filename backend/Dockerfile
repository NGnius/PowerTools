FROM ghcr.io/steamdeckhomebrew/holo-toolchain-rust:latest

RUN pacman -S --noconfirm cmake make clang git

ENTRYPOINT [ "/backend/entrypoint.sh" ]
