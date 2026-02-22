# justfile for Rnote

# Either 'true' or 'false'
ci := "false"
log_level := "debug"
build_folder := "_mesonbuild"
flatpak_app_folder := "_flatpak_app"
flatpak_repo_folder := "_flatpak_repo"
mingw64_prefix_path := "C:/msys64/mingw64"

[private]
linux_distr := `lsb_release -ds | tr '[:upper:]' '[:lower:]'`
[private]
sudo_cmd := "sudo"

export LANG := "C"
export RUST_BACKTRACE := "1"
export RUST_LOG := \
    "rnote=" + log_level + "," + \
    "rnote-cli=" + log_level + "," + \
    "rnote-engine=" + log_level + "," + \
    "rnote-compose=" + log_level
#export G_MESSAGES_DEBUG := "all"

default:
    just --list

prerequisites:
    #!/usr/bin/env bash
    set -euxo pipefail
    if [[ ('{{linux_distr}}' =~ 'fedora') ]]; then
        {{sudo_cmd}} dnf install -y \
            gcc gcc-c++ clang clang-devel python3 make cmake meson just git appstream gettext desktop-file-utils \
            shared-mime-info kernel-devel gtk4-devel libadwaita-devel alsa-lib-devel \
            appstream-devel
    elif [[ '{{linux_distr}}' =~ 'debian' || '{{linux_distr}}' =~ 'ubuntu' ]]; then
        {{sudo_cmd}} apt-get update
        {{sudo_cmd}} apt-get install -y \
            build-essential clang libclang-dev python3 make cmake meson just git appstream gettext desktop-file-utils \
            shared-mime-info libgtk-4-dev libadwaita-1-dev libasound2-dev libappstream-dev
    else
        echo "Unable to install system dependencies, unsupported distro."
        exit 1
    fi
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    export PATH="$HOME/.cargo/bin:$PATH"

prerequisites-flatpak: prerequisites
    #!/usr/bin/env bash
    set -euxo pipefail
    if [[ ('{{linux_distr}}' =~ 'fedora') ]]; then
        {{sudo_cmd}} dnf install -y \
            flatpak flatpak-builder
    elif [[ '{{linux_distr}}' =~ 'debian' || '{{linux_distr}}' =~ 'ubuntu' ]]; then
        {{sudo_cmd}} apt-get update
        {{sudo_cmd}} apt-get install -y \
            flatpak flatpak-builder
    else
        echo "Unable to install system dependencies, unsupported distro."
        exit 1
    fi
    flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
    flatpak install -y org.gnome.Platform//48 org.gnome.Sdk//48 org.freedesktop.Sdk.Extension.rust-stable//24.08 \
        org.freedesktop.Sdk.Extension.llvm19//24.08

prerequisites-dev: prerequisites
    #!/usr/bin/env bash
    set -euxo pipefail
    if [[ ('{{linux_distr}}' =~ 'fedora') ]]; then
        {{sudo_cmd}} dnf install -y \
            yamllint yq opencc-tools
    elif [[ '{{linux_distr}}' =~ 'debian' || '{{linux_distr}}' =~ 'ubuntu' ]]; then
        {{sudo_cmd}} apt-get update
        {{sudo_cmd}} apt-get install -y \
            yamllint yq opencc
    else
        echo "Unable to install system dependencies, unsupported distro."
        exit 1
    fi
    if [[ "{{ci}}" != "true" ]]; then
        ln -sf ../../build-aux/git-hooks/pre-commit.hook .git/hooks/pre-commit
    fi
    curl -L --proto '=https' --tlsv1.2 -sSf \
        https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
    cargo binstall -y cargo-nextest cargo-edit cargo-deny

# in MSYS2 shell
prerequisites-win:
    pacman -S --noconfirm \
        unzip git mingw-w64-x86_64-xz mingw-w64-x86_64-pkgconf mingw-w64-x86_64-gcc mingw-w64-x86_64-clang \
        mingw-w64-x86_64-toolchain mingw-w64-x86_64-autotools mingw-w64-x86_64-make mingw-w64-x86_64-cmake \
        mingw-w64-x86_64-meson mingw-w64-x86_64-diffutils mingw-w64-x86_64-desktop-file-utils \
        mingw-w64-x86_64-appstream mingw-w64-x86_64-gtk4 mingw-w64-x86_64-libadwaita mingw-w64-x86_64-angleproject
    mv /mingw64/lib/libpthread.dll.a /mingw64/lib/libpthread.dll.a.bak
    # We need to pin version : cairo before dwrite, gtk before dcomp
    # cairo : before 
    # https://github.com/msys2/MINGW-packages/commit/305ebda98c3041d9986d6fae498b45d2b2b9f4e8
    # Because the Dwrite win32 backend can't be used for text in multithreaded
    # context (see issue #1536 and upstream issues 
    # on mingw : https://github.com/msys2/MINGW-packages/issues/26222#issuecomment-3506563048
    # and on cairo's gitlab : https://gitlab.freedesktop.org/cairo/cairo/-/issues/886 
    # 
    # And because this also means that cairo will depend on the DLLmain
    # symbol being exported from gettext (removed in 
    # https://github.com/msys2/MINGW-packages/commit/3756eb5ceba81861751d26161a2ae6d980f715d3
    # with follow-up in 
    # https://github.com/msys2/MINGW-packages/commit/83eed715521d7d7c292d34fb80c29a720b534769#diff-a1d3a07941f44098abaaeef3440e303f11280e2165ff7e805fd50c7c38e8af13)
    # 
    # So
    # - we download the gettext runtime WITH the DLLmain symbol
    # - we install cairo/gtk4 with the pinned version. It will depend on DLLmain being 
    #   present pulling from the (now installed) version that includes i
    # - the rest (hopefully) will still work (if not, need to pin all dependencies in 
    # https://github.com/msys2/MINGW-packages/commit/83eed715521d7d7c292d34fb80c29a720b534769#diff-a1d3a07941f44098abaaeef3440e303f11280e2165ff7e805fd50c7c38e8af13
    # that we also use : which is a pretty long list !)
    # 
    # 1 : Get gettext before DLLmain removal
    wget -q https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-gettext-libtextstyle-0.26-1-any.pkg.tar.zst
    wget -q https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-gettext-runtime-0.26-1-any.pkg.tar.zst
    wget -q https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-gettext-tools-0.26-1-any.pkg.tar.zst
    pacman -U --noconfirm mingw-w64-x86_64-gettext-libtextstyle-0.26-1-any.pkg.tar.zst \
        mingw-w64-x86_64-gettext-runtime-0.26-1-any.pkg.tar.zst \
        mingw-w64-x86_64-gettext-tools-0.26-1-any.pkg.tar.zst
    # 2 : Get cairo and gtk4 pinned version
    # Also use a libadwaita version matching the gtk4 one
    wget -q https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-cairo-1.18.4-1-any.pkg.tar.zst
    wget -q https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-gtk4-4.18.6-3-any.pkg.tar.zst
    wget -q https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-libadwaita-1.7.7-1-any.pkg.tar.zst
    pacman -U --noconfirm mingw-w64-x86_64-cairo-1.18.4-1-any.pkg.tar.zst \
        mingw-w64-x86_64-gtk4-4.18.6-3-any.pkg.tar.zst \
        mingw-w64-x86_64-libadwaita-1.7.7-1-any.pkg.tar.zst
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    export PATH="$HOME/.cargo/bin:$PATH"

setup-dev *MESON_ARGS:
    meson setup \
        --prefix=/usr \
        -Dprofile=devel \
        -Dci={{ ci }} \
        {{ MESON_ARGS }} \
        {{ build_folder }}

setup-release *MESON_ARGS:
    meson setup \
        --prefix=/usr \
        -Dci={{ ci }} \
        {{ MESON_ARGS }} \
        {{ build_folder }}

# in MINGW64 shell
setup-win-installer installer_name="rnote-win-installer":
    meson setup \
        --prefix={{ mingw64_prefix_path }} \
        -Dprofile=default \
        -Dcli=true \
        -Dwin-installer-name={{ installer_name }} \
        -Dci={{ ci }} \
        {{ build_folder }}

clean:
    rm -rf {{ build_folder }}

configure *MESON_ARGS:
    meson configure {{ MESON_ARGS }} {{ build_folder }}

fmt-check:
    meson compile cargo-fmt-check -C {{ build_folder }}
    find . -name 'meson.build' | xargs meson format -q

fmt:
    cargo fmt
    find . -name 'meson.build' | xargs meson format -i

check:
    meson compile ui-cargo-check -C {{ build_folder }}
    meson compile cli-cargo-check -C {{ build_folder }}

lint:
    meson compile ui-cargo-clippy -C {{ build_folder }}
    meson compile cli-cargo-clippy -C {{ build_folder }}
    yamllint .

lint-dependencies:
    cargo deny check

build:
    meson compile ui-cargo-build -C {{ build_folder }}
    meson compile cli-cargo-build -C {{ build_folder }}

build-flatpak:
    flatpak-builder \
        --user \
        --repo={{ flatpak_repo_folder }} \
        --force-clean \
        {{ flatpak_app_folder}} \
        build-aux/com.github.flxzt.rnote.Devel.yaml

build-flatpak-bundle:
    flatpak build-bundle \
        {{ flatpak_repo_folder }} \
        com.github.flxzt.rnote.Devel.flatpak \
        com.github.flxzt.rnote.Devel \
        --runtime-repo=https://flathub.org/repo/flathub.flatpakrepo

build-win-installer: build
    meson compile rnote-gmo -C {{ build_folder }}
    meson compile build-installer -C {{ build_folder }}

install:
    meson install -C {{ build_folder }}

install-flatpak:
    flatpak-builder --user --install {{ flatpak_app_folder }} build-aux/com.github.flxzt.rnote.Devel.yaml

run-ui:
    {{ build_folder }}/target/debug/rnote

run-cli:
    {{ build_folder }}/target/debug/rnote-cli

run-flatpak:
    flatpak-builder --run {{ flatpak_app_folder }} build-aux/com.github.flxzt.rnote.Devel.yaml rnote

run-flatpak-no-backtrace:
    flatpak-builder --env=RUST_BACKTRACE=0 --run {{ flatpak_app_folder }} build-aux/com.github.flxzt.rnote.Devel.yaml rnote

test:
    meson test -C {{ build_folder }}
    meson compile cargo-test -C {{ build_folder }}

test-file-compatibility:
    {{ build_folder }}/target/debug/rnote-cli test \
        misc/file-tests/v0-5-5-test.rnote \
        misc/file-tests/v0-5-13-test.rnote \
        misc/file-tests/v0-6-0-test.rnote \
        misc/file-tests/v0-9-0-test.rnote

generate-docs:
    meson compile ui-cargo-doc -C {{ build_folder }}
    meson compile cli-cargo-doc -C {{ build_folder }}

check-outdated-dependencies:
    cargo upgrade --dry-run -vv

[doc('Regenerates the .pot file in the translations folder.
Note that all entries with strings starting and ending like this "@<..>@" must be removed,
they are templated variables and will be replaced in the build process of the app.
All changelog entries should be removed as well.')]
update-translations-template:
    meson compile rnote-pot -C {{ build_folder }}

update-translations:
    #!/usr/bin/env bash
    set -euxo pipefail

    # Regenerate 'zh_Hant' translation from 'zh_Hans'
    sed \
        -e 's|zh_Hans|zh_Hans\nzh_CN\nzh_SG|' \
        -e 's|zh_Hant|zh_Hant\nzh_HK\nzh_TW|' \
        "./crates/rnote-ui/po/LINGUAS" \
        | sort -uo "./crates/rnote-ui/po/LINGUAS"

    sed \
        -e 's|Language: zh_Hans|Language: zh_Hant|' \
        -e 's|Last-Translator:|Last-Translator: openCC converted|' \
        "./crates/rnote-ui/po/zh_Hans.po" \
        | opencc -c /usr/share/opencc/s2twp.json \
        -o "./crates/rnote-ui/po/zh_Hant.po"

create-tarball *MESON_DIST_ARGS:
    meson dist {{ MESON_DIST_ARGS }} -C {{ build_folder }}

generate-json-flatpak-manifest:
    yq -o=json build-aux/com.github.flxzt.rnote.Devel.yaml > build-aux/com.github.flxzt.rnote.Devel.json
