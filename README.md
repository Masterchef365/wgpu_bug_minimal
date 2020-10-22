# Minimal reproduction of a potential wgpu-rs bug 
This application is simply supposed to draw a grid of lines on the screen. On some Windows 10 machines using the DirectX12 backend, when the iced-wgpu backend is initialized before my custom backend the grid disappears. This does not happen in DirectX11, or Vulkan on Linux. On some machines the DirectX12/iced-wgpu combo does work; luckily it seems to be reproducible in a Windows 10 VM.

## Steps to reproduce
* Create a fresh Windows 10 x64 VM in VirtualBox (may not be necessary if your host machine exhibits the described behaviour, as some have)
* The VC++ redist might be needed; you can get it [here](https://support.microsoft.com/en-us/help/2977003/the-latest-supported-visual-c-downloads)
* Compile the project (transferring to the VM if necessary), and run it with the given arguments: `dx12 iced`. If the bug occurs, the window should remain blank. Passing just `iced` or just `dx12` should both display the grid, as in the former case the app defaults to DirectX11 and in the latter the `iced` backend is not initialized.