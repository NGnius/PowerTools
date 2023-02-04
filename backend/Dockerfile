FROM ghcr.io/steamdeckhomebrew/holo-toolchain-rust:latest

RUN pacman -S --noconfirm cmake make clang

ENTRYPOINT [ "/backend/entrypoint.sh" ]
