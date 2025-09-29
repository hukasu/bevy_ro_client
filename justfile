[group("asset_debug")]
act_debug:
    cargo run --bin act_debug --features="warning ragnarok_grf"

[group("asset_debug")]
gat_debug:
    cargo run --bin gat_debug --features="warning ragnarok_grf"

[group("asset_debug")]
pal_debug:
    cargo run --bin pal_debug --features="warning ragnarok_grf"

[group("asset_debug")]
rsm_debug:
    cargo run --bin rsm_debug --features="warning ragnarok_grf"

[group("asset_debug")]
rsw_debug:
    cargo run --bin rsw_debug --features="warning ragnarok_grf"

[group("asset_debug")]
spr_debug:
    cargo run --bin spr_debug --features="warning ragnarok_grf"

#[default]
[group("ci")]
default:
    @just format
    @just clippy

[group("ci")]
format:
    cargo fmt
    typos -w
    taplo fmt

[group("ci")]
clippy:
    @just ragnarok_act
    @just ragnarok_gat
    @just ragnarok_grf
    @just ragnarok_pal
    @just ragnarok_rsm
    @just ragnarok_rsw
    @just ragnarok_spr

    @just bevy_ragnarok_act
    @just bevy_ragnarok_grf
    @just bevy_ragnarok_pal
    @just bevy_ragnarok_rsm
    @just bevy_ragnarok_rsw
    @just bevy_ragnarok_spr

    @just bevy_ragnarok_quad_tree

[group("clippy")]
ragnarok_act $RUSTFLAGS="-Dwarnings":
    cargo clippy -p ragnarok_act --bins --lib --tests --no-default-features
    cargo clippy -p ragnarok_act --bins --lib --tests --no-default-features --features="warning"
    cargo clippy -p ragnarok_act --bins --lib --tests
    cargo clippy -p ragnarok_act --bins --lib --tests --all-features

[group("clippy")]
ragnarok_gat $RUSTFLAGS="-Dwarnings":
    cargo clippy -p ragnarok_gat --bins --lib --tests --no-default-features
    cargo clippy -p ragnarok_gat --bins --lib --tests --no-default-features --features="warning"
    cargo clippy -p ragnarok_gat --bins --lib --tests
    cargo clippy -p ragnarok_gat --bins --lib --tests --all-features

[group("clippy")]
ragnarok_grf $RUSTFLAGS="-Dwarnings":
    cargo clippy -p ragnarok_grf --bins --lib --tests --no-default-features
    cargo clippy -p ragnarok_grf --bins --lib --tests
    cargo clippy -p ragnarok_grf --bins --lib --tests --all-features

[group("clippy")]
ragnarok_pal $RUSTFLAGS="-Dwarnings":
    cargo clippy -p ragnarok_pal --bins --lib --tests --no-default-features
    cargo clippy -p ragnarok_pal --bins --lib --tests --no-default-features --features="warning"
    cargo clippy -p ragnarok_pal --bins --lib --tests
    cargo clippy -p ragnarok_pal --bins --lib --tests --all-features

[group("clippy")]
ragnarok_rsm $RUSTFLAGS="-Dwarnings":
    cargo clippy -p ragnarok_rsm --bins --lib --tests --no-default-features
    cargo clippy -p ragnarok_rsm --bins --lib --tests --no-default-features --features="warning"
    cargo clippy -p ragnarok_rsm --bins --lib --tests
    cargo clippy -p ragnarok_rsm --bins --lib --tests --all-features

[group("clippy")]
ragnarok_rsw $RUSTFLAGS="-Dwarnings":
    cargo clippy -p ragnarok_rsw --bins --lib --tests --no-default-features
    cargo clippy -p ragnarok_rsw --bins --lib --tests --no-default-features --features="warning"
    cargo clippy -p ragnarok_rsw --bins --lib --tests
    cargo clippy -p ragnarok_rsw --bins --lib --tests --all-features

[group("clippy")]
ragnarok_spr $RUSTFLAGS="-Dwarnings":
    cargo clippy -p ragnarok_spr --bins --lib --tests --no-default-features
    cargo clippy -p ragnarok_spr --bins --lib --tests --no-default-features --features="warning"
    cargo clippy -p ragnarok_spr --bins --lib --tests
    cargo clippy -p ragnarok_spr --bins --lib --tests --all-features

[group("clippy")]
bevy_ragnarok_act $RUSTFLAGS="-Dwarnings":
    cargo clippy -p bevy_ragnarok_act --bins --lib --tests --no-default-features
    cargo clippy -p bevy_ragnarok_act --bins --lib --tests --no-default-features --features="debug"
    cargo clippy -p bevy_ragnarok_act --bins --lib --tests
    cargo clippy -p bevy_ragnarok_act --bins --lib --tests --all-features

[group("clippy")]
bevy_ragnarok_grf $RUSTFLAGS="-Dwarnings":
    cargo clippy -p bevy_ragnarok_grf --bins --lib --tests --no-default-features
    cargo clippy -p bevy_ragnarok_grf --bins --lib --tests
    cargo clippy -p bevy_ragnarok_grf --bins --lib --tests --all-features

[group("clippy")]
bevy_ragnarok_pal $RUSTFLAGS="-Dwarnings":
    cargo clippy -p bevy_ragnarok_pal --bins --lib --tests --no-default-features
    cargo clippy -p bevy_ragnarok_pal --bins --lib --tests --no-default-features --features="debug"
    cargo clippy -p bevy_ragnarok_pal --bins --lib --tests
    cargo clippy -p bevy_ragnarok_pal --bins --lib --tests --all-features

[group("clippy")]
bevy_ragnarok_rsm $RUSTFLAGS="-Dwarnings":
    cargo clippy -p bevy_ragnarok_rsm --bins --lib --tests --no-default-features
    cargo clippy -p bevy_ragnarok_rsm --bins --lib --tests --no-default-features --features="debug"
    cargo clippy -p bevy_ragnarok_rsm --bins --lib --tests
    cargo clippy -p bevy_ragnarok_rsm --bins --lib --tests --all-features

[group("clippy")]
bevy_ragnarok_rsw $RUSTFLAGS="-Dwarnings":
    cargo clippy -p bevy_ragnarok_rsw --bins --lib --tests --no-default-features
    cargo clippy -p bevy_ragnarok_rsw --bins --lib --tests --no-default-features --features="debug"
    cargo clippy -p bevy_ragnarok_rsw --bins --lib --tests
    cargo clippy -p bevy_ragnarok_rsw --bins --lib --tests --all-features

[group("clippy")]
bevy_ragnarok_spr $RUSTFLAGS="-Dwarnings":
    cargo clippy -p bevy_ragnarok_spr --bins --lib --tests --no-default-features
    cargo clippy -p bevy_ragnarok_spr --bins --lib --tests --no-default-features --features="debug"
    cargo clippy -p bevy_ragnarok_spr --bins --lib --tests
    cargo clippy -p bevy_ragnarok_spr --bins --lib --tests --all-features

[group("clippy")]
bevy_ragnarok_quad_tree $RUSTFLAGS="-Dwarnings":
    cargo clippy -p bevy_ragnarok_quad_tree --bins --lib --tests --no-default-features
    cargo clippy -p bevy_ragnarok_quad_tree --bins --lib --tests --no-default-features --features="debug"
    cargo clippy -p bevy_ragnarok_quad_tree --bins --lib --tests --no-default-features --features="reflect"
    cargo clippy -p bevy_ragnarok_quad_tree --bins --lib --tests
    cargo clippy -p bevy_ragnarok_quad_tree --bins --lib --tests --all-features
