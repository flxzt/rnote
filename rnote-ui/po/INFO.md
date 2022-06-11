# Translation information

- When regenerating `rnote.pot`, all entries that have @..@ need to be removed or made sure to not be translated. These are placeholders and will be replaced when building the project with meson.
- The changelog should not be translated, so the entries need to be removed as well. ( in `app.metainfo.xml.in` )
