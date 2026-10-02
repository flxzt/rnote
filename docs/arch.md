# Architecture

The codebase is separated into multiple crates that have specific purposes and separate concerns:

- `rnote-compose` : the base crate that is only responsible for supplying basic types needed for a drawing application.
    Things like shapes, paths, pen-path builders, etc. In this crate is also the implementation for how to render
    these primitives with `cairo` or rather the `piet` abstraction.
    The dependencies should be kept minimal here. 
- `rnote-engine` : the core crate of the drawing application.
    In it is the entire core logic of the drawing part of the Rnote application.

    It is categorized like this:
    - `rnote-engine/store` : an Entity-Component-System pattern is used there to hold all strokes
    that are produced by the user in a generational Vector and the methods that define the interactions with them.
    - `rnote-engine/document` : information about the entire document (it's dimensions, colors, ..)
    - `rnote-engine/fileformats` : dictates the current stable Rnote file format,
        and implements the methods required to load and save itself;
        additionally contains the code required to convert from and into other formats
        (notably Xournal++'s `.xopp` format and older versions of the Rnote file format).
    - `rnote-engine/pens` : The user always generates/interacts with strokes through what Rnote internally
        calls `pens`. For example the "Brush" pen produces pen paths, the "Shaper" pen produces geometric shapes,
        the `eraser` pen removes strokes, .. .
    - `rnote-engine/strokes` contain the definition of different types that can be generated or imported
        into the engine. There are "brush strokes", "shape strokes" but also vector and rasterized images
        are represented as a "stroke type".
    
    The main "Engine" type is responsible for keeping an undo-stack and utilizes the Clone-On-Write datastructure
    of the "store" to achieve that.

    There are also smaller utilities and features like the "Camera" which is responsible for the canvas viewport,
    "AudioPlayer" to play pen sounds when enabled, .. .

- `rnote-cli` : basic CLI frontend that takes the engine as dependency and uses the "clap" crate.
    Intended to be used by power-users for automating format conversions or exports and other miscellaneous tasks.
    But it also plays a role in verifying the stability of the file format - it's test subcommand 
    is used in the CI to check whether `.rnote` files in different versions can still be imported successfully.

- `rnote-ui` : the UI frontend built with Gtk4 and Libadwaita.
    Most of the code here is glib `Object`'s or Gtk `Widget`s.
    The application is represented by `RnApp`, the main application window by `RnAppWindow`
    and the Canvas by `RnCanvas`. The canvas has one instance per tab and holds the engine.
