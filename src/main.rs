mod godot;
use godot::api::{register_public_api_func0, register_public_api_func3};
use godot::array::GodotArray;
use godot::variant::{Variant, VariantType};
use godot::sysalloc::fast_exit;

use serde_json::Value as JValue;
use serde_json::json;
use solidcraft_engine::Session;
use solidcraft_engine::command_specs;

/// Build a solid box (length/width/height in mm) in a fresh session and return a
/// compact inspection array:
///   [ok, body_count, vertex_count, triangle_count,
///    minx, miny, minz, maxx, maxy, maxz]
fn box_inspect(l: f64, w: f64, h: f64) -> Vec<Variant> {
    let mut vars: Vec<Variant> = Vec::with_capacity(10);
    let mut session = Session::default();
    let mut ok = false;

    match session.execute("solid.box", &json!({ "length": l, "width": w, "height": h })) {
        Ok(v) => {
            if v.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                ok = true;
            }
        }
        Err(_) => {}
    }
    vars.push(Variant::new_bool(ok));

    let world = session.world_state();
    let n = world.bodies.len();
    vars.push(Variant::new_integer(n as i64));

    let mut verts = 0i64;
    let mut tris = 0i64;
    let mut bounds_min = (f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    let mut bounds_max = (f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    if n > 0 {
        let mesh = world.bodies[0].mesh();
        verts = mesh.positions.len() as i64;
        tris = mesh.triangles.len() as i64;
        let b = mesh.bounds();
        bounds_min = (b.min.x, b.min.y, b.min.z);
        bounds_max = (b.max.x, b.max.y, b.max.z);
    }
    vars.push(Variant::new_integer(verts));
    vars.push(Variant::new_integer(tris));
    vars.push(Variant::new_float(bounds_min.0));
    vars.push(Variant::new_float(bounds_min.1));
    vars.push(Variant::new_float(bounds_min.2));
    vars.push(Variant::new_float(bounds_max.0));
    vars.push(Variant::new_float(bounds_max.1));
    vars.push(Variant::new_float(bounds_max.2));

    vars
}

/// Default box (10 x 20 x 30 mm) -- the canonical SolidCraft example.
extern "C"
fn solid_default() -> Variant {
    let vars = box_inspect(10.0, 20.0, 30.0);
    GodotArray::create(&vars).to_variant()
}

/// Parameterised box: length / width / height in mm.
extern "C"
fn solid_box(l: f64, w: f64, h: f64) -> Variant {
    let vars = box_inspect(l, w, h);
    GodotArray::create(&vars).to_variant()
}

/// How many command specs does the engine expose?
extern "C"
fn solid_command_count() -> Variant {
    let n = command_specs().len() as i64;
    Variant::new_integer(n)
}

pub fn main() {
    println!("solidcraft-guest starting");
    // Read the Variant-size note once so the linker keeps its section.
    let _ = unsafe { core::ptr::read_volatile(&godot::variant::SANDBOX_VARIANT_SIZE.size) };
    register_public_api_func0(
        "solid_default",
        solid_default,
        "Array",
        "",
    );
    register_public_api_func3(
        "solid_box",
        solid_box,
        "Array",
        "float length, float width, float height",
    );
    register_public_api_func0(
        "solid_command_count",
        solid_command_count,
        "int",
        "",
    );
    // Halting suspends the VM with the guest's state intact so the host can
    // call the registered API (the same pattern the MuJoCo guest uses).
    fast_exit();
}
