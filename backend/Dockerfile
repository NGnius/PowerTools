FROM ghcr.io/steamdeckhomebrew/holo-toolchain-rust:latest

RUN pacman -S --noconfirm cmake make

ENTRYPOINT [ "/backend/entrypoint.sh" ]
