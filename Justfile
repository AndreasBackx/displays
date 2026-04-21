set shell := ["zsh", "-cu"]

root := justfile_directory()
included_crates := "displays_types displays_logical_types displays_physical_types displays_windows_common displays_logical_linux displays_logical_windows displays_physical_linux_sys displays_physical_linux_logind displays_physical_linux displays_physical_windows displays displays_py displays_astal"

# TODO: Add these as a pre-commit hook or CI validation step so crate-local doc and LICENSE symlinks stay in sync.
sync-doc-links crate:
    mkdir -p "{{root}}/{{crate}}/docs"
    rm -rf "{{root}}/{{crate}}/docs/fragments"
    mkdir -p "{{root}}/{{crate}}/docs/fragments"
    for fragment in "{{root}}"/docs/readme/fragments/*.md; do ln -sfn "../../../${fragment#{{root}}/}" "{{root}}/{{crate}}/docs/fragments/$(basename "$fragment")"; done
    for fragment in "{{root}}"/docs/readme/fragments/examples/*.md; do ln -sfn "../../../${fragment#{{root}}/}" "{{root}}/{{crate}}/docs/fragments/$(basename "$fragment")"; done
    ln -sfn "../LICENSE" "{{root}}/{{crate}}/LICENSE"

readme crate:
    just sync-doc-links "{{crate}}"
    "{{root}}/scripts/generate-readme" "{{root}}/{{crate}}" "{{root}}/docs/readme/README.tpl" "{{root}}/{{crate}}/README.md"

root-readme:
    "{{root}}/scripts/generate-markdown" "{{root}}/docs/readme/root/README.src.md" "{{root}}/README.md"

readmes:
    for crate in {{included_crates}}; do just readme "$crate"; done
    just root-readme
