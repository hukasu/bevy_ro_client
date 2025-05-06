use criterion::{criterion_group, criterion_main, Criterion};

use bevy::render::{renderer::initialize_renderer, settings::WgpuSettings};
use wgpu::{
    BackendOptions, Backends, Instance, InstanceDescriptor, InstanceFlags, RequestAdapterOptions,
};

fn create_render_device(c: &mut Criterion) {
    c.bench_function("create_render_device", |b| {
        b.iter(|| {
            let instance = Instance::new(&InstanceDescriptor {
                backends: Backends::all(),
                flags: InstanceFlags::empty(),
                backend_options: BackendOptions::from_env_or_default(),
            });
            let (_render_device, _render_queue, _render_adapter_info, _render_adapter) =
                futures::executor::block_on(initialize_renderer(
                    &instance,
                    &WgpuSettings {
                        ..Default::default()
                    },
                    &RequestAdapterOptions::default(),
                ));
        })
    });
}

criterion_group!(benches, create_render_device);
criterion_main!(benches);
