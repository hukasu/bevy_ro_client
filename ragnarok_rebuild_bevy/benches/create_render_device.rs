use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};

use bevy::render::{
    renderer::{initialize_renderer, WgpuWrapper},
    settings::WgpuSettings,
};
use wgpu::{Backends, Instance, InstanceDescriptor, InstanceFlags, RequestAdapterOptions};

fn create_render_device(c: &mut Criterion) {
    c.bench_function("create_render_device", |b| {
        b.iter(|| {
            let instance = Arc::new(WgpuWrapper::new(Instance::new(InstanceDescriptor {
                backends: Backends::all(),
                flags: InstanceFlags::empty(),
                dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
                gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
            })));
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
