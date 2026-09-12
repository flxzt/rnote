# Create a Release

- Update PO template with `just update-translations-template` and wait for Weblate to apply it
- Update the German translation and commit/push in the Weblate interface
- Merge the translation PR
- Let the Github Action update the "Chinese Traditional" translation from "Chinese Simplified"
- Reset the Weblate repo
- Update Wust dependencies by running `cargo update` and update dependencies in flatpak dev YAML manifest
- Update the flatpak dev JSON manifest from YAML with `just generate-json-flatpak-manifest`
- Commit the changes and push
- Release commit: Update the changelog in the appdata file, replace all version numbers in the project and build,
    install and run all tests:
    - `just install`
    - `just lint`
    - `just test`
    - `just test-file-compatibility`
- Wait for CI to run successfully
- Create a release with tag `vX.Y.Z` on Github - the installer and tarball will be created by Github Actions CD
- For Flathub:
    - Create a new release branch, update the Flathub flatpak manifest with the new tarball download URL and
        update the dependencies. Create a PR, and wait for completion of the Flathub builder CI.
    - Create the new beta PR as well by copying the updated manifest from the new release branch
    - Merge both beta and release PRs.
        Optionally publish the builds on Flathub's runner web interface manually for a faster release

# Create a Tarball Manually

Create a tarball by running:

```bash
just create-tarball
```

This recipe uses the `meson dist` command.
Pass the `--no-tests` flag to skip tests and building when it has been made sure that it would build successfully.

```bash
just create-tarball --no-tests
```

A source tarball and checksum file should now be in directory `_mesonbuild/meson-dist/`.

# Create a Beta Release for Flathub

- Do the same first steps as for a regular release
- Add the suffix `-beta.<x>` to the app version suffix in meson and commit this to main.
- Checkout and rebase/merge the `beta` branch with `main`.
- Make a beta release on Github just like a regular one, but with `-beta.<x>` appended to the tag.
- In the Flathub app repo, all releases need to be made with a PR, so checkout a new branch from `beta` and update the
    flatpak manifest.
- Then create a PR on Github against `beta` and merge it as soon as the flathub runner runs successfully.

# Install a Previous Version Flatpak

To view all released flatpak versions:

```bash
flatpak remote-info --log flathub com.github.flxzt.rnote
```

To roll back and pin/unpin:

```bash
sudo flatpak update --commit=<version-hash> com.github.flxzt.rnote 
flatpak mask com.github.flxzt.rnote
flatpak mask --remove com.github.flxzt.rnote
```

# Flatpak Devel Manifest

A manifest `.json` is maintained in addition to the `.yaml`, because Gnome Builder currently only supports JSON Flatpak
manifests. To generate the JSON from YAML, [yq](https://github.com/mikefarah/yq) is used. Run:

```bash
just generate-json-flatpak-manifest
```

# Translations

Regenerate the .pot file in the translations folder.
Note that all entries with strings starting and ending like this "@<..>@" must be removed,
they are templated variables and will be replaced in the build process of the app.
All changelog entries should be removed as well.

```bash
just update-translations-template
```

The PO files can then be updated with:

```bash
meson compile rnote-update-po -C _mesonbuild
```

Usually Weblate updates them automatically, so this is not really needed unless one would want to reset and overwrite
the translation files on Weblate.

Before merging the weblate translation PR, the Weblate repository should be locked. Then merge the PR, reset and then
unlock the weblate repo to resynchronize with upstream.

## Update the "Chinese Traditional" Translation (zh_Hant.po) Manually

The package providing the `opencc` tool (`opencc-tools` on Fedora) needs to be installed to regenerate the translation.

To autogenerate chinese traditional translation (`zh_Hant`) from chinese simplified (`zh_Hans`)
use the following recipe:

```bash
just update-translations
```

This is discussed in [issue 220](https://github.com/flxzt/rnote/issues/220)

# Check outdated dependencies

Install [cargo-edit](https://github.com/killercup/cargo-edit)

Show outdated dependencies with:

```bash
just check-outdated-dependencies
```

With this output, update the dependencies manually in `Cargo.toml` where it makes sense.
