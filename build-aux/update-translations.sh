#!/usr/bin/env bash

script_dir=$(dirname "$0")

if ! command -v opencc &> /dev/null
then
    echo "The 'opencc' tool needs to be installed and present in PATH"
    exit 1
fi


# Regenerate 'zh_Hant' translation from 'zh_Hans'
sed \
    -e 's|zh_Hans|zh_Hans\nzh_CN\nzh_SG|' \
    -e 's|zh_Hant|zh_Hant\nzh_HK\nzh_TW|' \
    "${script_dir}/../crates/rnote-ui/po/LINGUAS" \
    | sort -uo "${script_dir}/../crates/rnote-ui/po/LINGUAS"

sed \
    -e 's|Language: zh_Hans|Language: zh_Hant|' \
    -e 's|Last-Translator:|Last-Translator: openCC converted|' \
    "${script_dir}/../crates/rnote-ui/po/zh_Hans.po" \
    | opencc -c /usr/share/opencc/s2twp.json \
       -o "${script_dir}/../crates/rnote-ui/po/zh_Hant.po"
