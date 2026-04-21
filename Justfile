set shell := ["zsh", "-cu"]

root := justfile_directory()
included_crates := "displays_types displays_logical_types displays_physical_types displays_windows_common displays_logical_linux displays_logical_windows displays_physical_linux_sys displays_physical_linux_logind displays_physical_linux displays_physical_windows displays"

readme crate:
    "{{root}}/scripts/generate-readme" "{{root}}/{{crate}}" "{{root}}/docs/readme/README.tpl" "{{root}}/{{crate}}/README.md"

readmes:
    for crate in {{included_crates}}; do just readme "$crate"; done
