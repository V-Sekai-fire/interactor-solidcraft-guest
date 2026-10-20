mod godot;
use godot::api::{register_public_api_func0, register_public_api_func1, register_public_api_func3, register_public_api_func4};
use godot::array::GodotArray;
use godot::string::GodotString;
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
            ok = true;
            let _ = v;
        }
        Err(e) => println!("solid.box({l}, {w}, {h}) failed: {e}"),
    }
    let world = session.world_state();
    let n = world.bodies.len();
    vars.push(Variant::new_bool(ok && n > 0));
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

/// Build one primitive in a fresh session and return its display mesh:
///   [ok, triangle_count, positions, normals, indices]
/// positions and normals are flat float Arrays (x, y, z per vertex, mm) and
/// indices a flat int Array (three per triangle, counter-clockwise outside).
/// kind 0 = box (length a, width b, height c), 1 = cylinder (radius a,
/// height b), 2 = sphere (radius a), 3 = torus (major a, minor b).
extern "C"
fn solid_mesh(kind: i64, a: f64, b: f64, c: f64) -> Variant {
    let (cmd, params) = match kind {
        0 => ("solid.box", json!({ "length": a, "width": b, "height": c })),
        1 => ("solid.cylinder", json!({ "radius": a, "height": b })),
        2 => ("solid.sphere", json!({ "radius": a })),
        _ => ("solid.torus", json!({ "major": a, "minor": b })),
    };
    let mut session = Session::default();
    let ran = session.execute(cmd, &params);
    if let Err(e) = &ran {
        println!("{cmd} {params} failed: {e}");
    }
    let world = session.world_state();
    if ran.is_err() || world.bodies.is_empty() {
        return GodotArray::create(&[Variant::new_bool(false)]).to_variant();
    }
    let mut out: Vec<Variant> = vec![Variant::new_bool(true)];
    out.extend(body_mesh(&world.bodies[0]));
    GodotArray::create(&out).to_variant()
}

/// One body's mesh as [triangle_count, positions, normals, indices]: positions and normals are
/// flat float Arrays (x, y, z per vertex, mm), indices a flat int Array (three per triangle,
/// counter-clockwise outside). A game-sized mesh: chord tolerance 1.5% of the body (the
/// engine's display mesh uses 0.1%, ten times the triangles of a sphere for no visible gain).
fn body_mesh(mb: &solidcraft_engine::doc::ModelBody) -> Vec<Variant> {
    let mesh = match mb.body.tessellate((mb.body.size() * 0.015).max(0.05)) {
        Ok(m) => std::sync::Arc::new(m),
        Err(_) => mb.mesh(),
    };
    let (positions, normals, triangles) = (&mesh.positions, &mesh.normals, &mesh.triangles);
    let mut pos: Vec<Variant> = Vec::with_capacity(positions.len() * 3);
    for p in positions {
        pos.push(Variant::new_float(p.x));
        pos.push(Variant::new_float(p.y));
        pos.push(Variant::new_float(p.z));
    }
    let mut nrm: Vec<Variant> = Vec::with_capacity(normals.len() * 3);
    for n in normals {
        nrm.push(Variant::new_float(n.x));
        nrm.push(Variant::new_float(n.y));
        nrm.push(Variant::new_float(n.z));
    }
    let mut idx: Vec<Variant> = Vec::with_capacity(triangles.len() * 3);
    for t in triangles {
        for i in t {
            idx.push(Variant::new_integer(*i as i64));
        }
    }
    vec![
        Variant::new_integer(triangles.len() as i64),
        GodotArray::create(&pos).to_variant(),
        GodotArray::create(&nrm).to_variant(),
        GodotArray::create(&idx).to_variant(),
    ]
}

/// Run a SolidCraft command script, `{"commands": [{"command", "params"}, ...]}` (the format
/// `solidcraft-cli run` takes), in a fresh session and return every body it leaves:
///   [ok, message, bodies]   bodies = [[name, triangle_count, positions, normals, indices], ...]
/// On failure ok is false and message names the failing step; bodies is then empty.
extern "C"
fn solid_script(script: GodotString) -> Variant {
    let text = script.to_string();
    let fail = |m: String| {
        GodotArray::create(&[Variant::new_bool(false), Variant::new_string(&m), GodotArray::create(&[]).to_variant()])
            .to_variant()
    };
    let parsed: JValue = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => return fail(format!("script is not JSON: {e}")),
    };
    let mut session = Session::default();
    if let Err(e) = session.run_script(&parsed) {
        return fail(e.to_string());
    }
    let world = session.world_state();
    let mut bodies: Vec<Variant> = Vec::with_capacity(world.bodies.len());
    for mb in world.bodies.iter() {
        let mut b = vec![Variant::new_string(&mb.name)];
        b.extend(body_mesh(mb));
        bodies.push(GodotArray::create(&b).to_variant());
    }
    let msg = format!("{} bodies", bodies.len());
    GodotArray::create(&[Variant::new_bool(true), Variant::new_string(&msg), GodotArray::create(&bodies).to_variant()])
        .to_variant()
}

/// How many command specs does the engine expose?
extern "C"
fn solid_command_count() -> Variant {
    let n = command_specs().len() as i64;
    Variant::new_integer(n)
}

pub fn main() {
    println!("solidcraft-guest starting");
    // The engine catches command panics and reports only their message; print
    // where each one happened so a failing command names its source line.
    std::panic::set_hook(Box::new(|info| println!("solidcraft-guest panic: {info}")));
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
    register_public_api_func4(
        "solid_mesh",
        solid_mesh,
        "Array",
        "int kind, float a, float b, float c",
    );
    register_public_api_func1(
        "solid_script",
        solid_script,
        "Array",
        "String script",
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
