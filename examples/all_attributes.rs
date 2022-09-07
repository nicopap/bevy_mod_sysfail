use bevy::{prelude::*, transform::TransformSystem};
use bevy_mod_system_tools::sys_chain;

enum GizmoError {
    Error,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_system(drag_gizmo.before(TransformSystem::TransformPropagate))
        .add_system(
            place_gizmo
                .after(TransformSystem::TransformPropagate)
                .after(drag_gizmo),
        );
    app.update();
}

// type DifferentErrorHandlingType = SystemType!(
//   fn different_error_handling(
//       In(result): In<Result<(), GizmoError>>,
//       mut last_error_occurence: Local<HashMap<GizmoError, Duration>>,
//       time: Res<Time>,
//   )
// );
// fn different_error_handling(
//     In(result): In<Result<(), GizmoError>>,
//     mut last_error_occurence: Local<HashMap<GizmoError, Duration>>,
//     time: Res<Time>,
// ) {
//     // A different, custom handling of errors
// }

#[sys_chain(log)]
fn drag_gizmo() -> Result<(), ()> {
    let _ = Err(())?;
    Ok(())
}
#[sys_chain(log)]
fn place_gizmo() -> Result<(), ()> {
    let _ = Err(())?;
    Ok(())
}
// #[sys_chain(system(different_error_handling: DifferentErrorHandlingType))]
// #[sys_chain(ignore = Duration::from_sec(30))]
#[sys_chain(ignore)]
fn delete_gizmo() -> Option<()> {
    let _ = None?;
    Some(())
}
