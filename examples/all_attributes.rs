use bevy::prelude::*;
use bevy_mod_sysfail::macros::*;

use thiserror::Error;

#[derive(Error, Debug)]
enum GizmoError {
    #[error("A Gizmo error")]
    Error,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_system(drag_gizmo)
        .add_system(delete_gizmo.after(place_gizmo))
        .add_system(place_gizmo.after(drag_gizmo));
    app.update();
}

#[sysfail(log)]
fn drag_gizmo(time: Res<Time>) -> Result<(), anyhow::Error> {
    println!("drag time is: {}", time.seconds_since_startup());
    let _ = Err(GizmoError::Error)?;
    println!("This will never print");
    Ok(())
}

#[sysfail(log(level = "info"))]
fn place_gizmo() -> Result<(), &'static str> {
    let () = Result::<(), &'static str>::Ok(())?;
    println!("this line should actually show up");
    let _ = Err("Ah, some creative use of info logging I see")?;
    Ok(())
}

#[quick_sysfail]
fn delete_gizmo(time: Res<Time>) {
    println!("delete time is: {}", time.seconds_since_startup());
    let _ = None?;
    println!("This will never print");
}
