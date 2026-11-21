use crate::settings::Settings;
use crate::cell::Cell;
use crate::ray::Ray;

use glam::Vec3;
use scene::{Scene, camera::{Camera, CameraType::{Orthogonal, Perspective}}, geometry::{HasPose, Pose}};


/// Represents the canvas where everything is rendered
/// It keeps a vector of all the cells, and the settings of the render
#[derive(Debug,Clone)]
pub struct Canvas {
    cells : Vec<Cell>,
    pub settings : Settings,
}

impl Canvas {
    /// Creates a canvas from a [`Settings`]
    /// Sets the cells vector to the correct size with a default [`Cell`]
    pub fn new(settings : Settings) -> Canvas {
        Canvas{
            cells : vec![Cell::default(); settings.width * settings.height], 
            settings
        }
    }

    /// Get width
    pub fn width(&self) -> usize {
        self.settings.width
    }
    /// Get height
    pub fn height(&self) -> usize {
        self.settings.height
    }

    fn idx(&self, x: usize, y: usize) -> Option<usize> {
        (x<self.width() && y<self.height() && self.cells.len() > y * self.width() + x).then_some(y * self.width() + x)
    }

    /// Get cell at position (x,y) returns `None` if either cells is not big enough or (x,y) is too big for the width and height.
    pub fn get_cell(&self, x : usize, y : usize) -> Option<&Cell> {
        self.idx(x, y).map(|i| &self.cells[i])
    }

    /// See [`get_cell`]
    pub fn get_mut_cell(&mut self, x : usize, y : usize) -> Option<&mut Cell> {
        self.idx(x, y).map(|i| &mut self.cells[i])
    }

    /// Sets a cell at (x,y) returns true if succesful and false if the (x,y) given too big for cells
    pub fn set(&mut self, x : usize, y: usize, cell : Cell) -> bool {
        if let Some(i) = self.idx(x, y) {
            if i >= self.cells.len() {
                self.cells.resize_with(i + 1, Cell::default);
            }
            self.cells[i] = cell;
            true
        } else {
            false
        }
    }

    /// Creates a ray from a cell's position and a camera
    /// If the position is outside of width/height 
    fn get_ray(&self, camera : Camera, x : usize, y : usize) -> Option<Ray> {
        self.idx(x,y)?;

        let width = self.width();
        let height = self.height()*2; // Actual height is double because ascii is 2:1

        let u = (x as f32 + 0.5)/width as f32;
        let v = (2.*y as f32 + 1.)/height as f32;  // Use +1 instead of +0.5 to get middle as Cells are 2:1

        let x_screen = 2.*u - 1.;
        let y_screen = 1. - 2.*v; // Flip y so y points up and not down as is the case in image formats

        let aspect_ratio = self.width() as f32 / self.height() as f32;
        let scale = match camera.camera_type {
            Perspective{fov} => (fov*0.5).tan(),
            Orthogonal{ortho_height: _} => 1.,
        };

        let x_cam = x_screen * aspect_ratio * scale;
        let y_cam = y_screen * scale;

        let direction = match camera.camera_type {
            Perspective{fov:_} => (camera.forward() + camera.right() * x_cam + camera.up() * y_cam).normalize(),
            Orthogonal { ortho_height } => camera.pose.position + (x_screen * aspect_ratio * ortho_height * 0.5) * camera.right() + (y_screen * ortho_height * 0.5) * camera.up(),
        };

        Some(Ray {
            origin : camera.pose.position,
            direction
        }) 
    }

    /// Get all rays for a camera and for the width/height of the canvas
    pub fn get_rays(&self, camera : Camera) -> Vec<Ray> {
        let mut ray_array = Vec::new();
        for y in 0..self.height() {
            for x in 0..self.width() {
                ray_array.push(self.get_ray(camera, x, y).unwrap_or_default())
            }
        }
        ray_array
    }

    /// Renders a scene using the canvas setting and updating cells
    pub fn render_scene(&mut self, scene : Scene) {
    }
     
}



mod test {
    use super::*;

    fn print_rays(rays : Vec<Ray>, width : usize, height : usize){
        let min_z = rays.iter().map(|r| r.direction.dot(Vec3::Z)).min_by(f32::total_cmp).unwrap();
        let max_z = rays.iter().map(|r| r.direction.dot(Vec3::Z)).max_by(f32::total_cmp).unwrap();

        let min_y = rays.iter().map(|r| r.direction.dot(Vec3::Y)).min_by(f32::total_cmp).unwrap();
        let max_y = rays.iter().map(|r| r.direction.dot(Vec3::Y)).max_by(f32::total_cmp).unwrap();
        
        let min_x = rays.iter().map(|r| r.direction.dot(Vec3::X)).min_by(f32::total_cmp).unwrap();
        let max_x = rays.iter().map(|r| r.direction.dot(Vec3::X)).max_by(f32::total_cmp).unwrap();

        for y in 0..height {
            for x in 0..width {
                let r = (rays[y * width + x].direction.dot(Vec3::Z) - min_z) * 1./(max_z-min_z)*9.;
                print!("{r:.0?}")
            }
            print!("  ");
            for x in 0..width {
                let r = (rays[y * width + x].direction.dot(Vec3::Y) - min_y) * 1./(max_y-min_y)*9.;
                print!("{r:.0?}")
            }
            print!("  ");
            for x in 0..width {
                let r = (rays[y * width + x].direction.dot(Vec3::X) - min_x) * 1./(max_x-min_x)*9.;
                print!("{r:.0?}")
            }
            print!("  ");
            println!();
        }
        println!();
    }

    #[test]
    fn test_print_rays () {
        let (width, height) = (24, 12);
        let canvas = Canvas::new(Settings { width, height });
        let rays = canvas.get_rays(Camera { pose: Pose::default(), camera_type: Perspective { fov: 50. } });
        print_rays(rays, width, height);
        
        let rays = canvas.get_rays(Camera { pose: Pose::default(), camera_type: Perspective { fov: 30. } });
        print_rays(rays,width,height);

        let rays = canvas.get_rays(Camera { pose: Pose::default(), camera_type: Orthogonal { ortho_height : 1.} });
        print_rays(rays,width,height);
    }
}