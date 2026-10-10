<div align="center">

# connector

A Game Boy-centric frontend for the Anbernic RG SP, built for link play.

<a href="LICENSE"><img src="https://img.shields.io/github/license/mjkoo/connector?style=for-the-badge&labelColor=242429&color=4d4d57" height="36" alt="License"></a>
<a href="https://github.com/mjkoo/connector/releases"><img src="https://img.shields.io/github/downloads/mjkoo/connector/total?style=for-the-badge&labelColor=242429&color=4d4d57" height="36" alt="Downloads"></a>
<a href="CHANGELOG.md"><img src="https://img.shields.io/badge/Changelog-4d4d57?style=for-the-badge&labelColor=242429" height="36" alt="Changelog"></a>

</div>

---

> [!NOTE]
> connector is a derivative of [slot.](https://github.com/BrandonKowalski/slot) by
> Brandon T. Kowalski. It is not an official slot. release and is not affiliated with or
> endorsed by the slot. project. See [Upstream and credits](#upstream-and-credits).

# What is connector?

A Game Boy-centric frontend for the Anbernic RG SP, with support for GBA, GBC and GB titles
only. It starts from slot. and adds its own work, link play over Wi-Fi first.

> [!IMPORTANT]
> ROMs must be unzipped. connector does not read `.zip` or `.7z` files.

A user guide lives in [`site/`](site/index.html). Release notes are in the
[changelog](CHANGELOG.md).

## Moving from slot.

Copy the contents of a release's `connector-<tag>/` folder onto the card as you would a
slot. update.

- Your settings carry over: `Config/slot.state` becomes `Config/connector.state` on first
  boot.
- Games, saves, states, labels, BIOS files and wallpapers stay where they are.
- On first boot, slot.'s `System/slot` binary, its logs (`slot.log`, `slot-session.log`) and
  any `crash/slot.core` are removed. The log is now `connector.log`.

---

# What does it look like?

<p align="center">
<img src=".github/resources/screenshots/carousel.gif" width="48%" alt="Scrolling the cart carousel across the Game Boy Advance, Game Boy and Game Boy Color shelves">
<img src=".github/resources/screenshots/core-picker.gif" width="48%" alt="Flipping a cart to its circuit board to choose mGBA or gpSP">
</p>

<p align="center">
<img src=".github/resources/screenshots/link.gif" width="96.5%" alt="Two SPs racing in Mario Kart over the emulated link">
</p>

---

# Supported Devices

| Device | Supported | Since Version |
| --- | --- | --- |
| Anbernic RG SP | Yes | Always |
| Anbernic RG34XX | Untested | N/A |
| Anbernic RG34XXSP | Untested | N/A |
| Anbernic RG35XXSP | Yes | 1.5.0 |

The RG34XX will likely work from 1.4.0. The RG34XXSP may not map its sticks correctly.

If you have one of the Untested devices, or have tried connector on a device not listed, please [open an issue here](https://github.com/mjkoo/connector/issues). Please do not report connector problems to the slot. project.

---

# Upstream and credits

connector is a fork of [slot.](https://github.com/BrandonKowalski/slot) by
**Brandon T. Kowalski**. His frontend, its design, its UI and art, the AGS-102 sticker and
wireless adapter, the cart sounds (a recording of his childhood Game Boy) and most of the
code are the foundation this project is built on. Brandon must continue to receive credit
for that work. If you enjoy connector, consider
[supporting him on Ko-fi](https://ko-fi.com/kwallbmoc).

- connector tracks slot. and merges its improvements.
- connector is not affiliated with or endorsed by the slot. project. Please report problems
  with connector [here](https://github.com/mjkoo/connector/issues), never to slot.
- The user guide's Cart Studio links point to slot.'s Cart Studio, which belongs to the
  slot. project.

Other credits, carried over from slot.: emulation by mGBA and gpSP, BaseOS by pvaibhav, Open
Sans and Nerd Fonts Symbols by Ryan L McIntyre, and the panel mask derived from Gigaherz's
LCD3x. Core and shader licences ship in `System/licenses/`.

## Licence

connector is free software under the GNU General Public License, version 3 or later. See
[LICENSE](LICENSE).

- Copyright (C) 2026 Brandon T. Kowalski (slot.)
- Copyright (C) 2026 Maxwell Koo and contributors (connector's modifications)

Every release links to the source it was built from.

# AI Disclosure

slot.'s Rust frontend was put together by Claude Opus and reviewed by Brandon T. Kowalski.
connector's changes are also written with Claude and reviewed by its maintainers.
