<div align="center">
<img src="rnote-ui/data/icons/scalable/apps/rnote.svg" width="256"></img>
</div><br><br><br>

<div align="start">
    <a href="https://liberapay.com/flxzt/donate">
        <img alt="Donate using Liberapay" src="https://github.com/flxzt/rnote/blob/main/misc/assets/liberapay-donate-button.svg" width="75" height="25">
    </a>
    <a href="https://www.paypal.com/donate?hosted_button_id=LQ9Q4868GKQGG">
        <img src="https://github.com/flxzt/rnote/blob/main/misc/assets/paypal-donate-button.png" alt="Donate with PayPal" width="75" height="25"/>
    </a>
</div><br>


# Rnote
> Sketch and take handwritten notes.  

Rnote is an open-source vector-based drawing app for sketching, handwritten notes and to annotate documents and pictures. Targeted at students, teachers and those who own a drawing tablet, it provides features like PDF and picture import and export, an infinite canvas and an adaptive UI for big and small screens.

Written in Rust and GTK4.

**Features**  
- Adaptive UI focused on stylus input
- Pressure-sensitive stylus input with different and configurable stroke styles
- Draw many different shapes with the shape tool
- Move, rotate, resize and modify existing content with the selection tool
- Different document expand layouts ( fixed pages, continuous vertical, infinite in every direction )
- Customizable background colors, patterns, sizes
- Customizable page format
- (optional) pen sounds
- Reconfigurable stylus button shortcuts
- An integrated workspace browser for quick access to related media files
- Drag & drop, clipboard support
- PDF, bitmap and SVG image import
- Save and load the documents in the native `.rnote` file format
- Document and selection export to SVG, PDF
- Autosave, printing

**Disclaimer**  
The file format is still unstable. It might change and break compatibility between versions.

## Website
Rnote has a project website: [rnote.flxzt.net](https://rnote.flxzt.net/)

## Installation
Rnote is available as a flatpak on Flathub:

<br><div align="start">
<a href='https://flathub.org/apps/details/com.github.flxzt.rnote'><img width="256" alt='Download on Flathub' src='https://flathub.org/assets/badges/flathub-badge-en.png'/></a>
</div><br>

**Downgrading**  
Because the file format still is unstable, downgrading to a specific version might be necessary and can be done with:
| version | command                                                                                                                |
| ------- | ---------------------------------------------------------------------------------------------------------------------- |
| v0.4.0  | `sudo flatpak update --commit=2ee585842334ad976802f08a1952c3fdc40f6f3afe2e056f3597fe4a029d54d2 com.github.flxzt.rnote` |
| v0.3.5  | `sudo flatpak update --commit=34115ec5896cbe1b7c1b7a589ec2b6da45e9fcbd81ae53c665c08f2fc42bb52f com.github.flxzt.rnote` |
| v0.2.5  | `sudo flatpak update --commit=2036a51c8118a30eb4ceb2e16ba2f84fa8ca4dc814fb88d9424709380093a6c6 com.github.flxzt.rnote` |
| v0.1.6  | `sudo flatpak update --commit=ffb9781989704f3eb28910437bb26709357566a977178d5fb4ef1a2926edae8b com.github.flxzt.rnote` |

After downgrading, the version can be pinned or unpinned with:
```
$ flatpak mask com.github.flxzt.rnote
$ flatpak mask --remove com.github.flxzt.rnote
```

Then the documents can be exported as an SVG or PDF and can be re-imported into the newest version of Rnote.

## Screenshots
![main_window_dark](./rnote-ui/data/screenshots/main_window_dark.png)
![main_window_light](./rnote-ui/data/screenshots/main_window_light.png)
![pdf_annotation](./rnote-ui/data/screenshots/pdf_annotation.png)
![selection](./rnote-ui/data/screenshots/selection.png)

## Pitfalls & known issues
* Drag & Drop: Make sure Rnote has permissions to the locations you are dragging files from. Can be granted in Flatseal (a Flatpak permissions manager)
* odd location for current file: when the directory displayed in the header title is something like `/run/user/1000/../`, rnote does not have permissions to access the directory.
    Again, granting them in Flatseal fixes this issue.
* Stylus buttons move canvas / are not functional: make sure that the `xf86-input-wacom`, drivers on X11 and `libinput` on Wayland and `libwacom` are installed and loaded.

## Credits
- A huge thanks to the contributors, translators and to all that donated. You are the ones that help keep the project going!
- [Freesound](https://freesound.org/) is the source for the pen sounds. The individual sounds are credited in `sounds/Licenses.md`
- [Rough.js](https://roughjs.com/) provides the algorithms for implementation of Rnote's rough shapes.
- [Pizarra](https://pizarra.categulario.xyz/en/) is an innovative drawing app with advanced shaping and featuring an infinite zoom. It is a great inspiration of the architecture of Rnote. Go check it out!

## Translations

<a href="https://hosted.weblate.org/engage/rnote/">
<img src="https://hosted.weblate.org/widgets/rnote/-/repo/multi-auto.svg" alt="Translation status" />
</a><br><br>

A great way to contribute to the project without writing code is adding and maintaining a new translation language. The translations files are located in `rnote-ui/po/`.

 Creating translations for new languages or updating existing ones can be done in two ways:
- take `rnote.pot` and generate a new `.po` translation file from it, add the new translation language to `LINGUAS` and submit a PR.
- use [weblate](https://hosted.weblate.org/projects/rnote/repo/) for an easy way to translate in the browser without having to deal with git.


## Community
If you have any questions or want to start a general discussion, open a topic in the [Github Discussions](https://github.com/flxzt/rnote/discussions) section.  
There is also the [#rnote:matrix.org](https://matrix.to/#/#rnote:matrix.org) chat room.  

## File Format
The `.rnote` file format is a gzipped json file. It is (de)compressed with the `flate2` crate and (de)serialized with the `Serde` crate.

So far breaking changes in the format happened in versions:  
- `v0.2.0`
- `v0.3.0`
- `v0.4.0`
- `v0.5.0`

To be able to open and export older files that are incompatible with the newest version, look under **Installation** /**Downgrading** to install older versions of Rnote.

## Drawings created with Rnote
If you have drawn something cool in Rnote and want to share it, submit a PR so it can be showcased here. :)  

<div align="center" spacing="20px">
        <img alt="Pikachu" src="https://github.com/flxzt/rnote/blob/main/misc/drawings/pikachu.png" height="400">
        <img alt="Tree" src="https://github.com/flxzt/rnote/blob/main/misc/drawings/tree.svg" height="400">
        <img alt="Love" src="https://github.com/flxzt/rnote/blob/main/misc/drawings/love.png" height="400">
</div><br>


## Building
Build instructions for Linux are documented in [BUILDING.md](./BUILDING.md) and for other platforms [here](./misc/building)
