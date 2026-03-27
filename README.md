<p id="downloads" align="center">
	<img src="https://i.stack.imgur.com/TOfqL.png" height="120px"/>
	<h1 align="center">Xtreme Download Manager</h1>
</p>

<p align="center">
	<a href="https://github.com/juanmmorel/xdm/workflows/Java%20CI/badge.svg?branch=master"><img src="https://github.com/juanmmorel/xdm/workflows/Java%20CI/badge.svg?branch=master" alt="Java CI" /></a>
	<a href="https://camo.githubusercontent.com/278e057571a0481121b2d60490ff656fb8736a20/68747470733a2f2f696d672e736869656c64732e696f2f6769746875622f646f776e6c6f6164732f73756268726137342f78646d2f746f74616c2e737667"><img src="https://img.shields.io/github/downloads/juanmmorel/xdm/total.svg" alt="Github All Releases" /></a>
</p>

### [XDM Homepage](https://xtremedownloadmanager.com/ "XDM Homepage") ###

**Important: This project is currently migrating from C#/.NET to Rust for the core and from JavaScript to Python for tools.**

**X**treme **D**ownload **M**anager (XDM) is a powerful tool to increase download speeds up to 500%, save videos from popular video streaming websites, resume broken/dead downloads, schedule and convert downloads.<br>
XDM seamlessly integrates with Google Chrome, Mozilla Firefox Quantum, Opera, Vivaldi and other Chromium and Firefox based browsers, to take over downloads and saving streaming videos from web.

## Migration Status
- **Core (Rust):** Work in progress in `app/xdm-rust`.
- **Tools (Python):** Translation generator rewritten in Python in `translation-generator/`.
- **Legacy (.NET):** The C# implementation in `app/XDM/` is being deprecated.

## Building from source

### Rust Core
To build the new Rust core, you need the Rust toolchain (cargo) installed.
<pre>
cd app/xdm-rust
cargo build
</pre>

### Translation Generator (Python)
The translation tool is now available as a Python script.
<pre>
python3 translation-generator/translation_gen.py
</pre>

## Features
- Download files at maximum possible speed (5-6 times faster than conventional downloaders).
- XDM can save video from numerous video streaming sites.
- Works with all modern browsers on Windows, Linux and Mac OS X.
- Supports `HTTP`, `HTTPS`, `FTP` as well as video streaming protocols like `MPEG-DASH`, `Apple HLS`, and `Adobe HDS`.
- Video download, clipboard monitoring, automatic antivirus checking, scheduler, system shutdown on download completion.
- Resumes broken / dead downloads caused by connection problem, power failure or session expiration.

[//]: #ImageLinks
[01]: https://i.stack.imgur.com/s7ViA.jpg
[02]: https://i.stack.imgur.com/90TQO.jpg
[03]: https://i.stack.imgur.com/V5XF3.jpg
[04]: https://i.stack.imgur.com/aFyH5.png
[05]: https://i.stack.imgur.com/lmAr6.png
[06]: https://i.stack.imgur.com/H4yMj.png
[07]: https://i.stack.imgur.com/8ulBq.png
[08]: https://i.stack.imgur.com/Gfgae.jpg
[09]: https://i.stack.imgur.com/GlVDC.png
