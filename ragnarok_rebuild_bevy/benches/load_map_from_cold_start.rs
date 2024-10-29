use std::sync::Arc;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use bevy::{
    app::{App, AppExit, PluginGroup, ScheduleRunnerPlugin, Startup},
    asset::AssetApp,
    prelude::{Commands, EventWriter, Res, Resource, Trigger},
    render::{
        renderer::{initialize_renderer, RenderInstance, WgpuWrapper},
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    },
    window::WindowPlugin,
    DefaultPlugins,
};
use futures::executor::block_on;
use wgpu::{Backends, Instance, InstanceDescriptor, InstanceFlags, RequestAdapterOptions};

use ragnarok_rebuild_bevy::{
    assets::{
        grf,
        rsw::{LoadWorld, WorldLoaded},
    },
    RagnarokPlugin,
};

#[derive(Resource)]
struct MapToLoad(String);

fn load_map_from_cold_start(c: &mut Criterion) {
    // Might freeze if run with all of them uncommented
    let maps = [
        "prontera.rsw",
        "pay_dun00.rsw",
        "schg_dun01.rsw",
        "lighthalzen.rsw",
    ];

    let instance = Arc::new(WgpuWrapper::new(Instance::new(InstanceDescriptor {
        backends: Backends::all(),
        flags: InstanceFlags::empty(),
        dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
    })));
    let (render_device, render_queue, render_adapter_info, render_adapter) =
        block_on(initialize_renderer(
            &instance,
            &WgpuSettings {
                ..Default::default()
            },
            &RequestAdapterOptions::default(),
        ));

    let mut group = c.benchmark_group("load_map_from_cold_start");
    for map in maps {
        group.bench_with_input(BenchmarkId::from_parameter(map), map, |b, s| {
            b.iter(|| {
                start_app_and_load_map(
                    s,
                    RenderCreation::Manual(
                        render_device.clone(),
                        render_queue.clone(),
                        render_adapter_info.clone(),
                        render_adapter.clone(),
                        RenderInstance(instance.clone()),
                    ),
                )
            })
        });
    }
    group.finish();
}

fn start_app_and_load_map(map: &str, render_creation: RenderCreation) {
    let mut app = App::new();
    app.insert_resource(MapToLoad(map.to_owned()))
        .register_asset_source(
            bevy::asset::io::AssetSourceId::Default,
            bevy::asset::io::AssetSourceBuilder::default().with_reader(|| {
                let grf = grf::AssetReader::new(std::path::Path::new("data.grf")).unwrap();
                Box::new(grf)
            }),
        )
        .add_plugins((
            DefaultPlugins
                .build()
                .set(RenderPlugin {
                    render_creation,
                    synchronous_pipeline_compilation: false,
                })
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: bevy::window::ExitCondition::DontExit,
                    close_when_requested: true,
                }),
            ScheduleRunnerPlugin::default(),
        ))
        .add_plugins(RagnarokPlugin)
        .add_systems(Startup, start_map_load)
        .observe(end_test_on_map_load);
    app.run();
}

fn start_map_load(mut commands: Commands, map_to_load: Res<MapToLoad>) {
    commands.trigger(LoadWorld {
        world: map_to_load.0.clone().into(),
    });
}

fn end_test_on_map_load(_trigger: Trigger<WorldLoaded>, mut event_writer: EventWriter<AppExit>) {
    event_writer.send(AppExit::Success);
}

criterion_group!(benches, load_map_from_cold_start);
criterion_main!(benches);
