FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

# Install required libusb dependencies
RUN sudo apt-get update && \
    sudo apt-get install -y libudev-dev libusb-1.0.0-dev && \
    sudo rm -rf /var/lib/apt/lists/*
