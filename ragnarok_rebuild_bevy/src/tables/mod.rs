pub mod name_table;
pub mod system_sets;

use bevy::{
    app::Startup,
    asset::{AssetServer, Handle},
    prelude::{App, Commands, Deref, IntoScheduleConfigs, ReflectResource, Res, Resource},
    reflect::Reflect,
};

pub struct TablePlugins;

impl bevy::app::Plugin for TablePlugins {
    fn build(&self, app: &mut App) {
        app
            // Register Plugins
            .add_plugins(name_table::Plugin)
            // Init name tables
            .add_systems(
                Startup,
                init_indoor_rsw.in_set(system_sets::TableSystems::TableStartup),
            );
    }
}

#[derive(Debug, Resource, Reflect, Deref)]
#[reflect(Resource)]
pub struct IndoorRsw(Handle<name_table::NameTable>);

fn init_indoor_rsw(mut commands: Commands, asset_server: Res<AssetServer>) {
    let indoor_rsw = asset_server.load("data/indoorrswtable.txt");
    commands.insert_resource(IndoorRsw(indoor_rsw));
}
