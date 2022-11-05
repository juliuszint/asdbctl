=======================================
Apple Studio Display Brightness Control
=======================================
A small command line utility to get or set the brightness level for Apple
Studio Displays from Linux.

Getting started
---------------
The software is written in Rust and uses `libusb` in the background. There are
no binary releases, which means you will need to install Rust (or rewrite it in
C). In either case you will need to install `libusb` with your distros package
manager. So when Rust and libusb is setup:::

    git clone https://github.com/juliuszint/asdbctl.git && cd asdbctl
    cargo build --release
    sudo ./target/release/asdbctl get

The program will need root permissions to function properly.

Background
----------
Dumping the USB traffic on macOS with Wireshark and setting the proper filters
will show the USB Control transfers to set the brightness::

    bmRequestType: 0x21
    bRequest     : 0x9
    wValue       : 0x0301
    wIndex:      : 0x000c
    wLength      : 0x7

with this data package when setting it to the minimum brightness value::

    [ 0x01, 0x90, 0x01, 0x00, 0x00, 0x00, 0x00 ]

The 0x90 and 0x01 are the brightness value encoded with the least significant
byte first (LSB).

Its possible to operate the Studio Display in 3 different USB configurations
and Linux will use the first. This means that the USB interface number for
controlling the brightness is not ``0xc`` (extracted from the dump) but ``0x7``.

Related projects
----------------
Other projects that helped along the way were `LG-ultrafine-brightness`_ and
acdcontrol_.

.. _acdcontrol: https://github.com/yhaenggi/acdcontrol
.. _LG-ultrafine-brightness: https://github.com/ycsos/LG-ultrafine-brightness
