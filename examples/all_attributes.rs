use bevy::prelude::*;
use bevy_mod_sysfail::prelude::*;
use bevy_mod_sysfail::Dedup;

use thiserror::Error;

#[derive(Component)]
struct Foo;

#[derive(Error, Debug)]
enum GizmoError {
    #[error("A Gizmo error")]
    Error,
}

impl Dedup for GizmoError {
    type ID = ();

    fn identify(&self) {}
}

fn main() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::log::LogPlugin::default()))
        .add_systems(Update, (drag_gizmo, (delete_gizmo, place_gizmo)).chain());
    app.update();
}

#[sysfail(Log<GizmoError>)]
fn drag_gizmo(time: Res<Time>) {
    println!("drag time is: {}", time.elapsed_seconds());
    let _ = Err(GizmoError::Error)?;
    println!("This will never print");
}

#[sysfail(Log<&'static str, Info>)]
fn place_gizmo() {
    let () = Result::<(), &'static str>::Ok(())?;
    println!("this line should actually show up");
    let _ = Err("Ah, some creative use of info logging I see")?;
}

/// This also has some doc
#[sysfail(Ignore)]
fn delete_gizmo(time: Res<Time>, mut query: Query<&mut Transform>, foos: Query<Entity, With<Foo>>) {
    println!("delete time is: {}", time.elapsed_seconds());
    for foo in &foos {
        let mut trans = query.get_mut(foo)?;
        trans.translation += Vec3::Y;
    }
    let _ = Err(())?;
    println!("This will never print");
}
