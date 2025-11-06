use bevy::{
    DefaultPlugins,
    app::{App, PreUpdate, Startup},
    asset::{
        AssetApp, AssetServer, Assets, RenderAssetUsages,
        io::{AssetSourceBuilder, AssetSourceId},
    },
    camera::{Camera, Camera2d, Camera3d, ClearColorConfig, RenderTarget},
    color::{Color, Srgba, palettes},
    ecs::{
        bundle::Bundle,
        children,
        entity::Entity,
        error::{BevyError, Result},
        hierarchy::Children,
        observer::On,
        query::With,
        relationship::RelationshipTarget,
        resource::Resource,
        schedule::{IntoScheduleConfigs, common_conditions::resource_changed},
        spawn::{Spawn, SpawnRelated},
        system::{Commands, Query, Res, ResMut, Single},
    },
    feathers::{
        FeathersPlugins,
        controls::{
            ColorChannel, ColorSlider, ColorSliderProps, ColorSwatch, SliderBaseColor, SliderProps,
            checkbox, color_slider, color_swatch, slider,
        },
        dark_theme::create_dark_theme,
        theme::{ThemeBackgroundColor, ThemedText, UiTheme},
        tokens,
    },
    image::Image,
    input_focus::tab_navigation::TabGroup,
    light::DirectionalLight,
    math::{Vec2, Vec3, primitives::Plane3d},
    mesh::{Mesh3d, MeshBuilder, Meshable},
    pbr::MeshMaterial3d,
    render::render_resource::{TextureDimension, TextureFormat, TextureUsages},
    text::TextFont,
    transform::components::Transform,
    ui::{
        BackgroundColor, Checked, FlexDirection, InteractionDisabled, Node, PositionType, UiRect,
        UiTargetCamera, Val,
        widget::{Text, ViewportNode},
    },
    ui_widgets::{SliderPrecision, SliderRange, SliderStep, SliderValue, ValueChange, observe},
};
use bevy_asset::AssetEvent;
use bevy_ecs::{component::Component, message::MessageReader};
use bevy_ragnarok_spr::{Sprite, SpriteImages, SpritesManager, material::SprMaterial};
use bevy_render::storage::ShaderStorageBuffer;

fn main() {
    let mut app = App::new();

    app.register_asset_source(
        AssetSourceId::Default,
        AssetSourceBuilder::platform_default("example_assets", None),
    );
    app.add_plugins(DefaultPlugins);
    app.add_plugins(FeathersPlugins);

    app.add_plugins(bevy_ragnarok_spr::plugin::Plugin);
    app.add_plugins(bevy_ragnarok_pal::plugin::Plugin);

    app.insert_resource(UiTheme(create_dark_theme()));
    app.init_resource::<SprControls>();

    app.add_systems(Startup, setup);
    app.add_systems(
        PreUpdate,
        update_color_sliders.run_if(resource_changed::<SprControls>),
    );
    app.add_systems(PreUpdate, asset_load);

    app.run();
}

#[derive(Default, Resource)]
struct SprControls {
    image: u32,
    true_color: bool,
    tint: Srgba,
    flipped_uv: bool,
}

#[derive(Component)]
struct ImageSlider;

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    let mut camera_render_target = Image::new_uninit(
        Default::default(),
        TextureDimension::D2,
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::all(),
    );
    camera_render_target.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    let render_target = images.add(camera_render_target);

    let ui_camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: 16,
                ..Default::default()
            },
        ))
        .id();
    let world_camera = commands
        .spawn((
            Camera3d::default(),
            Camera {
                clear_color: ClearColorConfig::Custom(Color::from(palettes::tailwind::GRAY_600)),
                target: RenderTarget::Image(render_target.into()),
                ..Default::default()
            },
            Transform::from_translation(Vec3::new(0., 0., 15.)).looking_at(Vec3::ZERO, Vec3::Y),
        ))
        .id();
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::new(0., 0., 5.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let mesh = asset_server.add(Plane3d::new(Vec3::NEG_Z, Vec2::splat(0.5)).mesh().build());

    commands.spawn((
        Mesh3d(mesh),
        Sprite::from(asset_server.load("poring.spr")),
        Transform::from_scale(Vec3::new(0.2, -0.2, -0.2)),
    ));

    setup_ui(&mut commands, ui_camera, world_camera);
}

fn setup_ui(commands: &mut Commands, ui_camera: Entity, world_camera: Entity) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        UiTargetCamera(ui_camera),
        children![
            (
                Node {
                    height: Val::Percent(100.),
                    width: Val::Px(384.),
                    padding: UiRect::all(Val::Px(4.)),
                    row_gap: Val::Px(6.),
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                ThemeBackgroundColor(tokens::WINDOW_BG),
                TabGroup::default(),
                children![
                    (
                        Text::new("Sprite controls"),
                        TextFont {
                            font_size: 24.,
                            ..Default::default()
                        }
                    ),
                    (
                        Text::new("Frame"),
                        TextFont {
                            font_size: 16.,
                            ..Default::default()
                        }
                    ),
                    (
                        Node {
                            height: Val::Px(24. * 2.),
                            row_gap: Val::Px(4.),
                            flex_direction: FlexDirection::Column,
                            ..Default::default()
                        },
                        children![
                            (
                                checkbox((), Spawn((Text::new("True color"), ThemedText))),
                                observe(true_color_value_change),
                            ),
                            (
                                ImageSlider,
                                slider(
                                    SliderProps {
                                        min: 0.,
                                        max: 1.,
                                        value: 0.,
                                    },
                                    (SliderStep(1.), SliderPrecision(0)),
                                ),
                                observe(image_slider_value_change)
                            )
                        ]
                    ),
                    (
                        Text::new("Tint"),
                        TextFont {
                            font_size: 16.,
                            ..Default::default()
                        }
                    ),
                    (
                        Node {
                            height: Val::Px(24. * 5.),
                            row_gap: Val::Px(4.),
                            flex_direction: FlexDirection::Column,
                            ..Default::default()
                        },
                        children![
                            (color_swatch(())),
                            make_color_slider(ColorChannel::Red),
                            make_color_slider(ColorChannel::Green),
                            make_color_slider(ColorChannel::Blue),
                            make_color_slider(ColorChannel::Alpha),
                        ]
                    ),
                    (
                        Text::new("Flip Uv"),
                        TextFont {
                            font_size: 16.,
                            ..Default::default()
                        }
                    ),
                    (
                        Node {
                            height: Val::Px(24.),
                            row_gap: Val::Px(4.),
                            flex_direction: FlexDirection::Column,
                            ..Default::default()
                        },
                        children![(
                            checkbox((), Spawn((Text::new("Uv flipped"), ThemedText))),
                            observe(flipped_uv_value_change),
                        )]
                    )
                ]
            ),
            (
                Node {
                    width: Val::Percent(100.),
                    ..Default::default()
                },
                ViewportNode::new(world_camera)
            )
        ],
    ));
}

fn make_color_slider(channel: ColorChannel) -> impl Bundle {
    (
        color_slider(ColorSliderProps { value: 1., channel }, ()),
        observe(color_slider_value_change),
    )
}

fn true_color_value_change(
    event: On<ValueChange<bool>>,
    mut commands: Commands,
    sprite: Single<Entity, With<MeshMaterial3d<SprMaterial>>>,
    mut sprite_manager: SpritesManager,
    mut sprite_controls: ResMut<SprControls>,
) -> Result<()> {
    if event.value {
        commands.entity(event.source).insert(Checked);
    } else {
        commands.entity(event.source).remove::<Checked>();
    }
    sprite_controls.true_color = event.value;
    sprite_controls.image = 0;

    commands.run_system_cached(update_image_slider_range);

    sprite_manager
        .change_image(*sprite, 0, event.value)
        .map_err(BevyError::from)
}

fn image_slider_value_change(
    event: On<ValueChange<f32>>,
    mut commands: Commands,
    sprite: Single<Entity, With<MeshMaterial3d<SprMaterial>>>,
    mut sprite_manager: SpritesManager,
    mut sprite_controls: ResMut<SprControls>,
) -> Result<()> {
    sprite_controls.image = event.value as u32;

    commands
        .entity(event.source)
        .insert(SliderValue(event.value));

    sprite_manager
        .change_image(*sprite, sprite_controls.image, sprite_controls.true_color)
        .map_err(BevyError::from)
}

fn color_slider_value_change(
    event: On<ValueChange<f32>>,
    sprite: Single<Entity, With<MeshMaterial3d<SprMaterial>>>,
    mut sprite_manager: SpritesManager,
    color_sliders: Query<&ColorSlider>,
    swatch: Single<&Children, With<ColorSwatch>>,
    mut background_colors: Query<&mut BackgroundColor>,
    mut sprite_controls: ResMut<SprControls>,
) -> Result<()> {
    let Ok(color_slider) = color_sliders.get(event.source) else {
        unreachable!("Must be a ColorSlider.");
    };
    match color_slider.channel {
        ColorChannel::Red => sprite_controls.tint.red = event.value,
        ColorChannel::Green => sprite_controls.tint.green = event.value,
        ColorChannel::Blue => sprite_controls.tint.blue = event.value,
        ColorChannel::Alpha => sprite_controls.tint.alpha = event.value,
        _ => unreachable!("Not a valid channel."),
    }

    let swatch_child = swatch.collection()[0];
    let Ok(mut bg) = background_colors.get_mut(swatch_child) else {
        unreachable!("First child of a swatch must have BackgroundColor.");
    };
    bg.0 = sprite_controls.tint.into();

    sprite_manager
        .change_tint(*sprite, sprite_controls.tint.into())
        .map_err(BevyError::from)
}

fn flipped_uv_value_change(
    event: On<ValueChange<bool>>,
    mut commands: Commands,
    sprite: Single<Entity, With<MeshMaterial3d<SprMaterial>>>,
    mut sprite_manager: SpritesManager,
    mut sprite_controls: ResMut<SprControls>,
) -> Result<()> {
    if event.value {
        commands.entity(event.source).insert(Checked);
    } else {
        commands.entity(event.source).remove::<Checked>();
    }
    sprite_controls.flipped_uv = event.value;

    sprite_manager
        .change_uv_flip(*sprite, sprite_controls.flipped_uv)
        .map_err(BevyError::from)
}

fn update_color_sliders(
    mut commands: Commands,
    color_sliders: Query<(Entity, &ColorSlider, &mut SliderBaseColor)>,
    sprite_controls: Res<SprControls>,
) {
    for (entity, color_slider, mut slider_base_color) in color_sliders {
        slider_base_color.0 = sprite_controls.tint.into();
        let value = match color_slider.channel {
            ColorChannel::Red => sprite_controls.tint.red,
            ColorChannel::Green => sprite_controls.tint.green,
            ColorChannel::Blue => sprite_controls.tint.blue,
            ColorChannel::Alpha => sprite_controls.tint.alpha,
            _ => unreachable!("Invalid color channel."),
        };
        commands.entity(entity).insert(SliderValue(value));
    }
}

fn asset_load(mut commands: Commands, mut asset_events: MessageReader<AssetEvent<SpriteImages>>) {
    for event in asset_events.read() {
        if matches!(event, AssetEvent::LoadedWithDependencies { id: _ }) {
            commands.run_system_cached(update_image_slider_range);
        }
    }
}

fn update_image_slider_range(
    mut commands: Commands,
    sprite: Single<&Sprite>,
    image_slider: Single<Entity, With<ImageSlider>>,
    sprite_controls: Res<SprControls>,
    sprite_images_assets: Res<Assets<SpriteImages>>,
    shader_storage_buffers: Res<Assets<ShaderStorageBuffer>>,
) {
    let sprite = sprite.into_inner();

    let Some(sprite_images) = sprite_images_assets.get(sprite.id()) else {
        unreachable!("SpriteImages must be available.");
    };

    let Some(shader_storage_buffer) = (if sprite_controls.true_color {
        shader_storage_buffers.get(sprite_images.true_color_sprites_uvs.id())
    } else {
        shader_storage_buffers.get(sprite_images.indexed_sprites_uvs.id())
    }) else {
        unreachable!("Uvs shader buffer must be valid.");
    };

    let Some((length, has_value)) = shader_storage_buffer.data.as_ref().map(|data| {
        let length = data.len() / SprMaterial::UV_STRIDE;
        let has_value = if length > 1 {
            true
        } else {
            data.ne(&[0; SprMaterial::UV_STRIDE])
        };
        (length, has_value)
    }) else {
        unreachable!("ShaderStorageBuffer can't be empty.");
    };

    if length == 1 && !has_value {
        commands.entity(*image_slider).insert(InteractionDisabled);
    } else {
        commands
            .entity(*image_slider)
            .remove::<InteractionDisabled>();
    }
    commands.entity(*image_slider).insert((
        SliderValue(0.),
        SliderRange::from_range(0f32..=(length as f32)),
    ));
}
