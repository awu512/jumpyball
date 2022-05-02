#![allow(dead_code)]
use frenderer::camera::Camera;
use frenderer::types::*;
use frenderer::{Engine, MousePos, Key, Result, WindowSettings};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

// GAME SETTINGS
const DT: f64 = 1.0 / 60.0; // time step

fn new_world(
    engine: &mut Engine,
    settings: PlayerSettings, 
    level_name: &str,
    start: Vec3, 
    end: Vec3
) -> Result<World, Box<dyn std::error::Error>> {

    let camera = Camera::look_at(
        Vec3::zero(),
        Vec3::zero(),
        Vec3::new(0., 1., 0.),
    );

    let player_tex = engine.load_texture(std::path::Path::new("content/sphere.png"))?;
    let player_mesh = engine.load_textured(std::path::Path::new("content/sphere.obj"))?;
    let player_model = engine.create_textured_model(player_mesh, vec![player_tex]);

    let level_tex = engine.load_texture(std::path::Path::new(&format!("content/{level_name}.png")))?;
    let level_mesh = engine.load_textured(std::path::Path::new(&format!("content/{level_name}.obj")))?;
    let level_model = engine.create_textured_model(level_mesh, vec![level_tex; 2]);

    let bounding_boxes = BoundingBox::from_file(&format!("content/{level_name}_bb.txt")).unwrap();

    let gem_tex = engine.load_texture(std::path::Path::new("content/gem.png"))?;
    let gem_mesh = engine.load_textured(std::path::Path::new("content/gem.obj"))?;
    let gem_model = engine.create_textured_model(gem_mesh, vec![gem_tex]);

    let world = World {
        camera,
        camera_control: OrbitCamera::new(),
        player: Player {
            settings,
            trf: Similarity3::new(start, Rotor3::identity(), 1.),
            model: player_model,
            vy: 0.,
            jump_count: 0,
        },
        level: Level {
            trf: Similarity3::new(Vec3::zero(), Rotor3::identity(), 3.),
            model: level_model,
            bounding_boxes,
        },
        start,
        goal: Goal {
            trf: Similarity3::new(end, Rotor3::identity(), 1.),
            model: gem_model,
            anim_counter: 50
        },
    };

    Ok(world)
}

struct PlayerSettings {
    radius: f32,
    velocity: f32,
    gravity: f32
}

struct BoundingBox {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    min_z: f32,
    max_z: f32,
}

impl BoundingBox {
    fn new(min_x: f32, max_x: f32, min_y: f32, max_y: f32, min_z: f32, max_z: f32) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
            min_z,
            max_z
        }
    }

    fn from_file(filepath: &str) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);

        let mut boxes: Vec<Self> = vec![];

        for line in reader.lines() {
            let unwrap = line.unwrap();
            let split: Vec<&str> = unwrap.split(' ').collect::<Vec<&str>>();
            let cast: Vec<f32> = split.iter().map(|s| s.parse::<f32>().unwrap()).collect();
            boxes.push(Self {
                min_x: cast[0],
                max_x: cast[1],
                min_y: cast[2],
                max_y: cast[3],
                min_z: cast[4],
                max_z: cast[5],
            })
        }

        Ok(boxes)
    }
}

struct Sphere {
    pos: Vec3,
    r: f32,
}

fn player_touching_end(p:&Player, g:&Goal) -> bool {
    let s: Sphere = Sphere { 
        pos: p.trf.translation,
        r: p.settings.radius,
    };

    let b: BoundingBox = BoundingBox { 
        min_x: g.trf.translation.x - 0.5,
        max_x: g.trf.translation.x + 0.5,
        min_y: g.trf.translation.y - 1.,
        max_y: g.trf.translation.y + 1.,
        min_z: g.trf.translation.z - 0.5,
        max_z: g.trf.translation.z + 0.5,
    };

    let closest = Vec3::new(
        s.pos.x.clamp(b.min_x, b.max_x),
        s.pos.y.clamp(b.min_y, b.max_y),
        s.pos.z.clamp(b.min_z, b.max_z),
    );

    let dist = closest - s.pos;

    dist.mag() <= s.r
}

fn handle_collision(p: &mut Player, b: &BoundingBox) {
    let s: Sphere = Sphere { 
        pos: p.trf.translation,
        r: p.settings.radius,
    };

    let closest = Vec3::new(
        s.pos.x.clamp(b.min_x, b.max_x),
        s.pos.y.clamp(b.min_y, b.max_y),
        s.pos.z.clamp(b.min_z, b.max_z),
    );

    let dist = closest - s.pos;

    if dist.mag() < s.r {
        let rest = (s.pos - closest).normalized() * (s.r-(s.pos - closest).mag());

        if rest.x.abs() > rest.y.abs() && rest.x.abs() > rest.z.abs() {
            p.trf.translation.x += rest.x;
        } else if rest.y.abs() > rest.z.abs() {
            p.trf.translation.y += rest.y;
            p.vy = 0.;
            p.jump_count = 0;
        } else {
            p.trf.translation.z += rest.z;
        }
    }
}

pub struct OrbitCamera {
    pub pitch: f32,
    pub yaw: f32,
    pub distance: f32,
    player_pos: Vec3,
}

impl OrbitCamera {
    fn new() -> Self {
        Self {
            pitch: 0.,
            yaw: 0.,
            distance: 5.,
            player_pos: Vec3::zero(),
        }
    }

    fn update(&mut self, events: &frenderer::Input, player: &Player) {
        let MousePos { x: dx, y: dy } = events.mouse_delta();
        self.pitch += (DT * dy) as f32 / 10.0;
        self.pitch = self.pitch.clamp(-PI / 4.0, PI / 4.0);

        self.yaw += (DT * dx) as f32 / 10.0;
        // self.yaw = self.yaw.clamp(-PI / 4.0, PI / 4.0);
        // self.distance += events.key_axis(Key::Up, Key::Down) * 5.0 * DT as f32;
        self.player_pos = player.trf.translation;
        // self.player_rot = player.trf.rotation;
        // TODO: when player moves, slightly move yaw towards zero
    }

    fn update_camera(&self, c: &mut Camera) {
        // The camera should point at the player (you could transform
        // this point to make it point at the player's head or center,
        // or at point in front of the player somewhere, instead of
        // their feet)
        let at = self.player_pos;
        // And rotated around the player's position and offset backwards
        let camera_rot = Rotor3::from_euler_angles(0.0, self.pitch, self.yaw);
        // self.player_rot = camera_rot;
        let offset = camera_rot * Vec3::new(0.0, 0.0, -self.distance);
        let eye = self.player_pos + offset;
        // To be fancy, we'd want to make the camera's eye an object
        // in the world whose rotation is locked to point towards the
        // player, and whose distance from the player is locked, and
        // so on---so we'd have player OR camera movements apply
        // accelerations to the camera which could be "beaten" by
        // collision.
        *c = Camera::look_at(eye, at, Vec3::unit_y());
    }
}

struct Player {
    settings: PlayerSettings,
    trf: Similarity3,
    model: Rc<frenderer::renderer::textured::Model>,
    vy: f32,
    jump_count: u8,
}

struct Level {
    trf: Similarity3,
    model: Rc<frenderer::renderer::textured::Model>,
    bounding_boxes: Vec<BoundingBox>,
}

struct Goal {
    trf: Similarity3,
    model: Rc<frenderer::renderer::textured::Model>,
    anim_counter: u16
}
struct World {
    camera: Camera,
    camera_control: OrbitCamera,
    player: Player,
    level: Level,
    start: Vec3,
    goal: Goal,
}
struct Flat {
    trf: Similarity3,
    model: Rc<frenderer::renderer::flat::Model>,
}

impl frenderer::World for World {
    fn update(&mut self, input: &frenderer::Input, _assets: &mut frenderer::assets::Assets) {

        // JUMP MECHANICS
        if input.is_key_pressed(Key::Space) && self.player.jump_count < 2 {
            self.player.vy = 3. * self.player.settings.velocity;
            self.player.jump_count += 1;
        }
      
        // CALCULATE PLAYER MOVEMENT
        let rotation = Rotor3::from_euler_angles(0.0, 0.0, self.camera_control.yaw);
        self.player.vy += self.player.settings.gravity;
        let move_vec = rotation * Vec3::new(
            input.key_axis(Key::D, Key::A),
            self.player.vy,
            input.key_axis(Key::S, Key::W)
        );

        // EXECUTE PLAYER MOVEMENT
        self.player.trf.translation.x += self.player.settings.velocity * move_vec.x;
        self.player.trf.translation.y += move_vec.y;
        self.player.trf.translation.z += self.player.settings.velocity * move_vec.z;
      
        // GROUND CHECK
        if self.player.trf.translation.y < self.player.settings.radius {
            self.player.trf.translation = self.start;
            self.player.vy = 0.;
            self.player.jump_count = 0;
        }

        // ADJUST ROTATION BASED ON JUMP
        let rot_mult = if self.player.jump_count == 0 {
            1.
        } else if self.player.jump_count == 1 {
            0.5
        } else {
            2.
        };
      
        // HANDLE COLLISION
        for b in &self.level.bounding_boxes {
            handle_collision(&mut self.player, b);
        }

        // CHECK END OF LEVEL
        if player_touching_end(&self.player, &self.goal) {
            dbg!("You win!"); // move to next level
        }

        // ROTATE PLAYER
        self.player.trf.prepend_rotation(Rotor3 {
            s: 1.,
            bv: Bivec3 {
                xy: (move_vec.x / (2.*PI*self.player.settings.radius)) * rot_mult,
                xz: 0.,
                yz: -(move_vec.z / (2.*PI*self.player.settings.radius)) * rot_mult,
            },
        });

        // ADJUST CAMERA
        self.camera_control.update(input, &self.player);
        self.camera_control.update_camera(&mut self.camera);

        // ANIMATE GOAL
        if self.goal.anim_counter >= 200 { self.goal.anim_counter = 0 }
        let dy: f32 = if self.goal.anim_counter < 100 { -0.005 } else { 0.005 };
        self.goal.anim_counter += 1;
        self.goal.trf.translation.y += dy;
    }

    fn render(
        &mut self,
        _a: &mut frenderer::assets::Assets,
        rs: &mut frenderer::renderer::RenderState,
    ) {
        rs.set_camera(self.camera);

        rs.render_textured(self.player.model.clone(), self.player.trf, 0);
        rs.render_textured(self.level.model.clone(), self.level.trf, 1);
        rs.render_textured(self.goal.model.clone(), self.goal.trf, 2);
    }
} 
fn main() -> Result<()> {
    frenderer::color_eyre::install()?;

    let mut engine: Engine = Engine::new(WindowSettings::default(), DT);

    let settings = PlayerSettings {
        radius: 1.,
        velocity: 0.2,
        gravity: -0.03
    };

    let world = new_world(
        &mut engine,
        settings,
        "level_1",
        Vec3::new(-12.75, 10., 11.25),
        Vec3::new(-15.0, 10.0, -15.0)
    ).unwrap();

    engine.play(world)
}