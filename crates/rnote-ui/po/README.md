# Rnote Translation Information
- When regenerating `rnote.pot`, all entries that have @..@ need to be removed or made sure to not be translated. These
    are placeholders and will be replaced when building the project with meson.
- The changelog should not be translated, so those entries ( from `app.metainfo.xml.in` ) need to be removed as well.
- Certain chinese locales are not listed in the `LINGUAS` file. This is because they are only symlinks to `zh_hans` and
    because weblate is configured to ignore these translations, it will always remove them from `LINGUAS`. However in
    practice this is not an issue, because meson's `rnote-gmo` target still compiles them to `.mo` files, and for
    windows the installer script includes the translations based on enumerating the files present in the `po` directory,
    not based on the locale strings present in `LINGUAS`. We just need to make sure we keep it this way.
    (discussed in more detail in [#838](https://github.com/flxzt/rnote/pull/838))
