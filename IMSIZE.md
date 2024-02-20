`/crates/rnote-engine/src/engine/import.rs` :

```rs
    pub fn generate_vectorimage_from_bytes(
        &self,
        pos: na::Vector2<f64>,
        bytes: Vec<u8>,
    ) -> oneshot::Receiver<anyhow::Result<VectorImage>> {
```

```rs
    pub fn generate_bitmapimage_from_bytes(
        &self,
        pos: na::Vector2<f64>,
        bytes: Vec<u8>,
    ) -> oneshot::Receiver<anyhow::Result<BitmapImage>> {
```
In the past implementation, this was the function to send the `format` to the next step in the process, with the `resize` bool as well.

But now we have two of these functions for svg vs not svg.

----
Layout : in `document`, enum `Layout` to find

`crates/rnote-engine/src/strokes/bitmapimage.rs`

For the raterized image

BEWARE : we are interested in `from_image_bytes` and this function is also called elsewhere so defaults needed 


And 

`crates/rnote-engine/src/strokes/vectorimage.rs/`

BEWARE : we are interested in `from_svg_str` and this function is also called elsewhere so we need to have defaults as well

For now we MAY have a correct implementation for the bitmap image only


---
Incorrect thing for the resize caus eonly would work for a single page

--- 
Change the algo : do with respect to the viewport size instead !

---
other imports to see ?
- pdf imports : laisses sans resize ds ts les cas (imports, pas ctrl + v !) 
- stroke : always to size
- text : seems to always be 1/2 page by default

faire un `ctrl+v` special pour respecter les bordures VERTICALES uniquements s'appliquant a un peu plus d'elements, SI BESOIN, sans test pour verifier si l'on 


For text : temporary `set_text_width` to force the size ?

For stroke : 

--- 
what is going on with `import_generated_content` and the widget resize 
- called from `imexport` for `load_in_vectorimage_bytes`, `load_in_bitmapimage_bytes`, `load_in_pdf_bytes`

Actually just calculating document sizes and width (total NOT format related)
But in this you also find related functions for pages

---
Ok so possible to set the custom paste thing for shift ctrl + v but we can't do that for imported images from the user's dragging things inside the rnote windows

---
separate logic when pasting a single element from the clipboard 
-> change the `ctrl + c` part to into into a single element ?
    maybe not, would be possible to do a resize after the element is pasted ? But only for the special ctrl + v case ofc
-> changge at the `ctrl + v` part ? YES, need to adapt it to stroke content (that also includes figures but also a little bit more)



Drag and drop part :
+ modifiers

Called from `open_file_w_dialogs`. Common with opening file out right. All of the logic is in `crates/rnote-ui/src/canvas/mod.rs`
```rs
/// drop target
```


---
image fail : from `gen_image` in `crates/rnote-engine/src/render.rs`


--- 
general_regular_cursor_picker_menubutton for the regular cursor
Shows up in `regular_cursor` from the `RnCanvas`

file `crates/rnote-ui/src/canvas/mod.rs`

---
Transformable : trait from the `crates/rnote-compose/src/transform/transformable.rs`

Call the `fn scale(&mut self, scale: na::Vector2<f64>)` thing on each element.

But we have to see what happens on multiple selections
`resize_lock_aspectratio`

```rs
pub struct SelectorConfig {
    #[serde(rename = "style")]
    pub style: SelectorStyle,
    #[serde(rename = "resize_lock_aspectratio")]
    pub resize_lock_aspectratio: bool,
}
```

Call `handle_pen_event_down` with modifier keys ? From what ? contains `PenEvent`
All of the logic is from `crates/rnote-ui/src/canvas/input.rs`


---
also keyboard events. Can we set them to do a global `ctrl+a` depending on the context that is much more forgiving ?
Issue : still must NOT override the rest (that is not activated for any text field and the like)

Idea : activated when the TOOLBAR is in focus as well


--- 
compare the pen event with modifiers and the rest

When are keyboard events captured for pen events ? What is the secret sauce there making everything work ?

```rs
            // Pointer controller
            let pen_state = Cell::new(PenState::Up);
            self.pointer_controller.connect_event(clone!(@strong pen_state, @weak obj as canvas => @default-return glib::Propagation::Proceed, move |_, event| {
                let (propagation, new_state) = super::input::handle_pointer_controller_event(&canvas, event, pen_state.get()); // event ?
                pen_state.set(new_state);
                propagation
            }));
```
Gets a pure event and tries
```rs
    let gdk_modifiers = event.modifier_state(); 
```
to get the modifiers as well and their states
```
maybe connect_actions_notify ?
```


```rs
        let appwindow_drop_target = self.imp().drop_target.connect_drop(
            clone!(@weak self as canvas, @weak appwindow => @default-return false, move |_, value, x, y| {
                let pos = (canvas.engine_ref().camera.transform().inverse() *
                    na::point![x,y]).coords;
                let mut accept_drop = false;
```

---
Need to capture both shift AND buttons from the pen if this is a dnd thing as well


```rs
let regular_cursor = gdk::Cursor::from_texture(
    &gdk::Texture::from_resource(
        (String::from(config::APP_IDPATH)
            + "icons/scalable/actions/cursor-dot-medium.svg")
            .as_str(),
    ),
    32, 
    32,
    gdk::Cursor::from_name("default", None).as_ref(),
);
```


Commit 
- captured shift + drag and drop
- now how to capture pen button + drag and drop 
    - do a connect up, connect down connect drop to set the drag and drop status and invert the relation ?
    - wip : some debug traces for stroke content and comments for 

Invert the thing. Activate true for dnd on `connect_enter`, false for dnd on `connect_leave` and take the option on `connect_drop` and reset it

We reverted it, we check with shift first then go on the next part for the stylus