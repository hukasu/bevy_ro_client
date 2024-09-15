pub mod name_table;

use bevy::{
    app::Startup,
    asset::{AssetServer, Handle},
    prelude::{Commands, Deref, ReflectResource, Res, Resource},
    reflect::Reflect,
};

pub struct TablePlugins;

impl bevy::app::Plugin for TablePlugins {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Register Plugins
            .add_plugins(name_table::Plugin)
            // Init name tables
            .add_systems(Startup, init_indoor_rsw);
    }
}

#[derive(Debug, Resource, Reflect, Deref)]
#[reflect(Resource)]
pub struct IndoorRsw(Handle<name_table::NameTable>);

fn init_indoor_rsw(mut commands: Commands, asset_server: Res<AssetServer>) {
    let indoor_rsw = asset_server.load("data/indoorrswtable.txt");
    commands.insert_resource(IndoorRsw(indoor_rsw));
}
