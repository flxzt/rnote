#!/usr/bin/env bash

if [[ "$1" == "-h" ]] || [[ "$1" == "--help" ]]; then
    echo "\
deploy-remote.sh

deploy the meson installation remotely

Usage: deploy-remote.sh <ssh-remote> <remote-password>

Note that it will install the files into the path with the configured prefix of the local build.
Clear your history after executing the script with 'history -c' to ensure the remote password does not get stored
in the bash history.
"
fi

remote=$1
remote_passwd=$2

tmp_path="/tmp/rnote-remote-deploy"
install_folder="build-install"
tmp_install="${tmp_path}/${install_folder}"
tmp_archive="${tmp_path}/archive.tar.gz"
remote_data_dir="/usr/share"

mkdir "${tmp_path} || true"
meson install --destdir "${tmp_install}" -C _mesonbuild
tar -zcf "${tmp_archive}" -C "${tmp_path}" "${install_folder}"
ssh "${remote}" "mkdir ${tmp_path} || true"
scp "${tmp_archive}" "${remote}:${tmp_archive}"
ssh "${remote}" "tar -xz --no-same-owner -f ${tmp_archive} --directory ${tmp_path}"
echo "${remote_passwd}" | ssh "${remote}" "sudo -S cp -r ${tmp_install}/* /"

# post-install steps
echo "${remote_passwd}" | ssh "${remote}" "sudo -S gtk-update-icon-cache ${remote_data_dir}/icons/hicolor"
echo "${remote_passwd}" | ssh "${remote}" "sudo -S glib-compile-schemas ${remote_data_dir}/glib-2.0/schemas"
echo "${remote_passwd}" | ssh "${remote}" "sudo -S update-desktop-database ${remote_data_dir}/applications"
echo "${remote_passwd}" | ssh "${remote}" "sudo -S update-mime-database ${remote_data_dir}/mime"
echo "${remote_passwd}" | ssh "${remote}" "sudo -S fc-cache -v -f"
