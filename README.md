# NVIDIA GPU Task Manager (Rust)

A lightweight GPU monitor built with Rust and egui, optimized for Linux hybrid-graphics laptops.

## Tested Environment
* **Hardware**: Lenovo Yoga Pro 9 (83DN)
* **CPU/GPU**: Core Ultra 9 185H, RTX 4070 Laptop (8GB)
* **OS**: Fedora Linux 43 (Workstation Edition)

## Key Features
* **Power Efficient**: Only polls NVML when the dGPU is active to save battery.
* **Process Tracking**: Maps NVML data to `/proc` for full command-line info and "ghost process" detection.
* **Process Management**: Integrated "Kill" functionality for GPU-bound processes.
* **PCI Control**: Toggle GPU power management states (`auto`/`on`) directly from the UI.

## Setup & Permissions
To allow PCI power control without root:
1. Create `/etc/udev/rules.d/99-nvidia-pwr.rules`:
   `ACTION=="add", SUBSYSTEM=="pci", ATTR{vendor}=="0x10de", ATTR{power/control}="*", MODE="0664", GROUP="video"`
2. Add your user to the group: `sudo usermod -aG video $USER`

## Usage
```bash
cargo run --release