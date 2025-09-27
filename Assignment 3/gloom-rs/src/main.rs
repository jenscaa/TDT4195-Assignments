// Uncomment these following global attributes to silence most warnings of "low" interest:
/*
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]
*/
extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod util;
mod mesh;
mod scene_graph;
mod toolbox;

use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;
use scene_graph::SceneNode;
use toolbox::simple_heading_animation;

// initial window size
const INITIAL_SCREEN_W: u32 = 800;
const INITIAL_SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //

// Get the size of an arbitrary array of numbers measured in bytes
// Example usage:  byte_size_of_array(my_array)
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
// Example usage:  pointer_to_array(my_array)
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
// Example usage:  size_of::<u64>()
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T, represented as a relative pointer
// Example usage:  offset::<u64>(4)
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()


// Creates a VAO and returns its id
unsafe fn create_vao(vertices: &Vec<f32>, indices: &Vec<u32>, colors: &Vec<f32>, normals: &Vec<f32>) -> u32 {
    // Creating and setting up a Vertex Array Object
    let mut vao_id: u32 = 0;
    gl::GenVertexArrays(1, &mut vao_id);
    gl::BindVertexArray(vao_id);

    // Creating a Vertex Buffer Object
    let mut vbo: u32 = 0;
    gl::GenBuffers(1, &mut vbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(vertices),
        pointer_to_array(vertices),
        gl::STATIC_DRAW,
    );

    // Enabling the Vertex Attributes
    let stride = 3 * size_of::<f32>();
    gl::EnableVertexAttribArray(0);
    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride as i32, offset::<f32>(0));

    // Creating an Index Buffer Object
    let mut ibo: u32 = 0;
    gl::GenBuffers(1, &mut ibo);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        byte_size_of_array(indices),
        pointer_to_array(indices),
        gl::STATIC_DRAW,
    );

    // Create a color buffer object
    let mut cbo: u32 = 0;
    gl::GenBuffers(1, &mut cbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, cbo);
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(colors),
        pointer_to_array(colors),
        gl::STATIC_DRAW,
    );

    // Enabling color attributes
    gl::EnableVertexAttribArray(1);
    gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, 0, ptr::null());

    // Create a normal buffer object
    let mut nbo: u32 = 0;
    gl::GenBuffers(1, &mut nbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, nbo);
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(normals),
        pointer_to_array(normals),
        gl::STATIC_DRAW,
    );

    // Enable normal attribute (location = 2)
    gl::EnableVertexAttribArray(2);
    gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());

    return vao_id;
}

unsafe fn draw_scene(
    node: &scene_graph::SceneNode,
    shader: &shader::Shader,
    view_projection_matrix: &glm::Mat4,
    transformation_so_far: &glm::Mat4,
) {
    
    let mut local_transform = glm::translate(&glm::identity(), &node.position);

    // Move pivot to reference point
    local_transform = glm::translate(&local_transform, &node.reference_point);

    // Apply rotations around Z, Y, X
    local_transform = glm::rotate(&local_transform, node.rotation.z, &glm::vec3(0.0, 0.0, 1.0));
    local_transform = glm::rotate(&local_transform, node.rotation.y, &glm::vec3(0.0, 1.0, 0.0));
    local_transform = glm::rotate(&local_transform, node.rotation.x, &glm::vec3(1.0, 0.0, 0.0));

    // Move pivot back
    local_transform = glm::translate(&local_transform, &-node.reference_point);

    // Combine model matrix with the scene's View Projection matrix
    let model_matrix = transformation_so_far * local_transform;
    let model_view_projection_matrix = view_projection_matrix * model_matrix;

    shader.activate();
    gl::UniformMatrix4fv(
        shader.get_uniform_location("u_model_view_projection"),
        1, gl::FALSE, model_view_projection_matrix.as_ptr(),
    );
    gl::UniformMatrix4fv(
        shader.get_uniform_location("u_model"),
        1, gl::FALSE, model_matrix.as_ptr(),
    );

    // Check if node is drawable, if so: set uniforms, bind VAO and draw VAO
    if node.vao_id > 0 && node.index_count > 0 {
        shader.activate();
        gl::UniformMatrix4fv(
            shader.get_uniform_location("u_transform"),
            1,
            gl::FALSE,
            model_view_projection_matrix.as_ptr(),
        );

        gl::BindVertexArray(node.vao_id);
        gl::DrawElements(gl::TRIANGLES, node.index_count, gl::UNSIGNED_INT, ptr::null());
        gl::BindVertexArray(0);
    }

    // Recurse
    for &child in &node.children {
        draw_scene(&*child, shader, view_projection_matrix, &model_matrix);
    }
}

fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize::new(INITIAL_SCREEN_W, INITIAL_SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Set up shared tuple for tracking changes to the window size
    let arc_window_size = Arc::new(Mutex::new((INITIAL_SCREEN_W, INITIAL_SCREEN_H, false)));
    // Make a reference of this tuple to send to the render thread
    let window_size = Arc::clone(&arc_window_size);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers.
        // This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        let mut window_aspect_ratio = INITIAL_SCREEN_W as f32 / INITIAL_SCREEN_H as f32;

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
        }

        let simple_shader = unsafe {
            shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.vert")
                .attach_file("./shaders/simple.frag")
                .link()
        };

        // let u_transform_loc = unsafe {
        //     let name = std::ffi::CString::new("u_transform").unwrap();
        //     gl::GetUniformLocation(simple_shader.program_id, name.as_ptr())
        // };

        // Load the terrain mesh from file
        let terrain = mesh::Terrain::load("./resources/lunarsurface.obj");

        // Terrain VAO
        let terrain_vao = unsafe { create_vao(&terrain.vertices, &terrain.indices, &terrain.colors, &terrain.normals) };
        let terrain_index_count = terrain.index_count;

        // Load the helicopter mesh from file
        let helicopter = mesh::Helicopter::load("./resources/helicopter.obj");

        // Create VAOs for each part
        let body_vao = unsafe { create_vao(&helicopter.body.vertices, &helicopter.body.indices, &helicopter.body.colors, &helicopter.body.normals) };
        let door_vao = unsafe { create_vao(&helicopter.door.vertices, &helicopter.door.indices, &helicopter.door.colors, &helicopter.door.normals) };
        let main_rotor_vao = unsafe { create_vao(&helicopter.main_rotor.vertices, &helicopter.main_rotor.indices, &helicopter.main_rotor.colors, &helicopter.main_rotor.normals) };
        let tail_rotor_vao = unsafe { create_vao(&helicopter.tail_rotor.vertices, &helicopter.tail_rotor.indices, &helicopter.tail_rotor.colors, &helicopter.tail_rotor.normals) };

        // Index counts for each part
        let body_index_count       = helicopter.body.index_count;
        let door_index_count       = helicopter.door.index_count;
        let main_rotor_index_count = helicopter.main_rotor.index_count;
        let tail_rotor_index_count = helicopter.tail_rotor.index_count;

        // Root node for scene
        let mut scene_root = SceneNode::new();

        // Terrain node
        let mut terrain_node = SceneNode::from_vao(terrain_vao, terrain_index_count);
        scene_root.add_child(&terrain_node);


        // // Helicopter root node
        // let mut helicopter_root = SceneNode::new();
        // terrain_node.add_child(&helicopter_root);

        // // Helicopter nodes
        // let mut body_node = SceneNode::from_vao(body_vao, body_index_count);
        // let mut door_node = SceneNode::from_vao(door_vao, door_index_count);
        // let mut main_rotor_node = SceneNode::from_vao(main_rotor_vao, main_rotor_index_count);
        // let mut tail_rotor_node = SceneNode::from_vao(tail_rotor_vao, tail_rotor_index_count);

        // helicopter_root.add_child(&body_node);
        // helicopter_root.add_child(&door_node);
        // helicopter_root.add_child(&main_rotor_node);
        // helicopter_root.add_child(&tail_rotor_node);

        // helicopter_root.print();

        // // Setting reference points for helicopter parts
        // body_node.reference_point = glm::vec3(0.0, 0.0, 0.0);
        // door_node.reference_point = glm::vec3(0.0, 0.0, 0.0);
        // main_rotor_node.reference_point = glm::vec3(0.0, 0.0, 0.0);
        // tail_rotor_node.reference_point = glm::vec3(0.35, 2.3, 10.4);

        const NUMBER_OF_HELICOPTERS: usize = 5;

        // Store handles to helicopter nodes
        let mut helicopter_roots: Vec<scene_graph::Node> = Vec::with_capacity(NUMBER_OF_HELICOPTERS);
        let mut main_rotor_nodes: Vec<scene_graph::Node> = Vec::with_capacity(NUMBER_OF_HELICOPTERS);
        let mut tail_rotor_nodes: Vec<scene_graph::Node> = Vec::with_capacity(NUMBER_OF_HELICOPTERS);
        let mut door_nodes: Vec<scene_graph::Node> = Vec::with_capacity(NUMBER_OF_HELICOPTERS);

        for _ in 0..NUMBER_OF_HELICOPTERS {
            // Helicopter root node
            let mut helicopter_root = SceneNode::new();
            terrain_node.add_child(&helicopter_root);

            // Helicopter parts
            let mut body_node = SceneNode::from_vao(body_vao, body_index_count);
            let mut door_node = SceneNode::from_vao(door_vao, door_index_count);
            let mut main_rotor_node = SceneNode::from_vao(main_rotor_vao, main_rotor_index_count);
            let mut tail_rotor_node = SceneNode::from_vao(tail_rotor_vao, tail_rotor_index_count);

            helicopter_root.add_child(&body_node);
            helicopter_root.add_child(&door_node);
            helicopter_root.add_child(&main_rotor_node);
            helicopter_root.add_child(&tail_rotor_node);

            // Reference points
            body_node.reference_point = glm::vec3(0.0, 0.0, 0.0);
            door_node.reference_point = glm::vec3(0.0, 0.0, 0.0);
            main_rotor_node.reference_point = glm::vec3(0.0, 0.0, 0.0);
            tail_rotor_node.reference_point = glm::vec3(0.35, 2.3, 10.4);

            // Save nodes for later animation
            helicopter_roots.push(helicopter_root);
            main_rotor_nodes.push(main_rotor_node);
            tail_rotor_nodes.push(tail_rotor_node);
            door_nodes.push(door_node);
        }



        // Used to demonstrate keyboard handling for exercise 2.
        let mut _arbitrary_number = 0.0;

        let mut cam_pos   = glm::vec3(0.0, 0.0, 0.0);
        let mut cam_yaw  : f32 = 0.0;
        let mut cam_pitch: f32 = 0.0;

        // let move_speed: f32 = 500.0;
        let move_speed: f32 = 100.0;
        let rot_speed : f32 = 2.5;
        let pitch_limit: f32 = std::f32::consts::FRAC_PI_2 - 0.01;


        // The main rendering loop
        let first_frame_time = std::time::Instant::now();
        let mut previous_frame_time = first_frame_time;
        loop {

            // Compute time passed since the previous frame and since the start of the program
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(previous_frame_time).as_secs_f32();
            previous_frame_time = now;

            // Handle resize events
            if let Ok(mut new_size) = window_size.lock() {
                if new_size.2 {
                    context.resize(glutin::dpi::PhysicalSize::new(new_size.0, new_size.1));
                    window_aspect_ratio = new_size.0 as f32 / new_size.1 as f32;
                    (*new_size).2 = false;
                    println!("Window was resized to {}x{}", new_size.0, new_size.1);
                    unsafe { gl::Viewport(0, 0, new_size.0 as i32, new_size.1 as i32); }
                }
            }

            // Compute camera orientation vectors from yaw + pitch
            let forward = glm::vec3(
                cam_yaw.sin() * cam_pitch.cos(),
                cam_pitch.sin(),
                -cam_yaw.cos() * cam_pitch.cos(),
            );
            let up = glm::vec3(0.0, 1.0, 0.0);
            let right    = glm::normalize(&glm::cross(&forward, &up));

            // Build view matrix
            let view: glm::Mat4 = glm::look_at(&cam_pos, &(cam_pos + forward), &up);

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {

                for key in keys.iter() {
                    match key {
                        // Move (WASD + Space / LShift)
                        VirtualKeyCode::W => {
                            cam_pos += forward * move_speed * delta_time;
                        }
                        VirtualKeyCode::S => {
                            cam_pos -= forward * move_speed * delta_time;
                        }
                        VirtualKeyCode::A => {
                            cam_pos -= right * move_speed * delta_time;
                        }
                        VirtualKeyCode::D => {
                            cam_pos += right * move_speed * delta_time;
                        }
                        VirtualKeyCode::Space => {
                            cam_pos += up * move_speed * delta_time;
                        }
                        VirtualKeyCode::LShift => {
                            cam_pos -= up * move_speed * delta_time;
                        }
                        // Rotate (arrow keys)
                        VirtualKeyCode::Left => {
                            cam_yaw -= rot_speed * delta_time;
                        }
                        VirtualKeyCode::Right => {
                            cam_yaw += rot_speed * delta_time;
                        }
                        VirtualKeyCode::Up => {
                            cam_pitch += rot_speed * delta_time;
                        }
                        VirtualKeyCode::Down => {
                            cam_pitch -= rot_speed * delta_time;
                        }

                        // Open/close door (X/Z)
                        VirtualKeyCode::X => {
                        for door in &mut door_nodes {
                            if door.position.z < 2.0 {
                                door.position.z += 0.2;
                            }
                        }
                        }
                        VirtualKeyCode::Z => {
                        for door in &mut door_nodes {
                            if door.position.z > 0.0 {
                                door.position.z -= 0.2;   
                            }
                        }
                        }


                        _ => {}
                    }
                }

                cam_pitch = cam_pitch.clamp(-pitch_limit, pitch_limit);
            }


            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {

                // == // Optionally access the accumulated mouse movement between
                // == // frames here with `delta.0` and `delta.1`

                *delta = (0.0, 0.0); // reset when done
            }

            // == // Please compute camera transforms here (exercise 2 & 3)

            // let model: glm::Mat4 = glm::translate(&glm::identity(), &glm::vec3(0.0, 0.0, -2.0));

            const FOVY_RAD: f32 = std::f32::consts::FRAC_PI_4;
            let projection: glm::Mat4 = glm::perspective(
                window_aspect_ratio,
                FOVY_RAD,
                1.0,
                2000.0,
            );

            // let transform: glm::Mat4 = projection * view * model;

            // let heading = simple_heading_animation(elapsed);

            // // Move the helicopter along the path (X/Z plane). Keep a fixed altitude.
            // helicopter_root.position.x = heading.x;
            // helicopter_root.position.y = 5.0;
            // helicopter_root.position.z = heading.z;

            // // Apply orientation to the helicopter (root), showing rotation on all three axes.
            // // Mapping: roll -> Z, yaw -> Y, pitch -> X (then drawn in Z→Y→X order).
            // helicopter_root.rotation.z = heading.roll;
            // helicopter_root.rotation.y = heading.yaw;
            // helicopter_root.rotation.x = heading.pitch;


            // // Spin the main rotor
            // main_rotor_node.rotation.y += 10.0 * delta_time;  

            // // Spin the tail rotor
            // tail_rotor_node.rotation.x += 15.0 * delta_time;

            // Animate all helicopters
            for (i, root) in helicopter_roots.iter_mut().enumerate() {
                let heading = simple_heading_animation(elapsed + i as f32 * 0.75);

                root.position.x = heading.x;
                root.position.y = 5.0;
                root.position.z = heading.z;

                root.rotation.z = heading.roll;
                root.rotation.y = heading.yaw;
                root.rotation.x = heading.pitch;
            }

            // Spin all rotors
            for main_rotor in &mut main_rotor_nodes {
                main_rotor.rotation.y += 10.0 * delta_time;
            }
            for tail_rotor in &mut tail_rotor_nodes {
                tail_rotor.rotation.x += 15.0 * delta_time;
            }

            unsafe {
                gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky
                //gl::ClearColor(1.0, 0.0, 1.0, 1.0); // magenta
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                simple_shader.activate();
                // gl::UniformMatrix4fv(u_transform_loc, 1, gl::FALSE, transform.as_ptr());

                // gl::BindVertexArray(terrain_vao);
                // gl::DrawElements(
                //     gl::TRIANGLES,
                //     terrain_index_count,
                //     gl::UNSIGNED_INT,
                //     std::ptr::null(),
                // );

                // gl::BindVertexArray(body_vao);
                // gl::DrawElements(gl::TRIANGLES, body_index_count, gl::UNSIGNED_INT, ptr::null());

                // gl::BindVertexArray(door_vao);
                // gl::DrawElements(gl::TRIANGLES, door_index_count, gl::UNSIGNED_INT, ptr::null());

                // gl::BindVertexArray(main_rotor_vao);
                // gl::DrawElements(gl::TRIANGLES, main_rotor_index_count, gl::UNSIGNED_INT, ptr::null());

                // gl::BindVertexArray(tail_rotor_vao);
                // gl::DrawElements(gl::TRIANGLES, tail_rotor_index_count, gl::UNSIGNED_INT, ptr::null());

                // gl::BindVertexArray(0);
                

                let u_light_pos_loc = simple_shader.get_uniform_location("u_lightPos");
                gl::Uniform3f(u_light_pos_loc, 0.8, -0.5, 0.6);

                let u_view_pos_loc = simple_shader.get_uniform_location("u_viewPos");
                gl::Uniform3f(u_view_pos_loc, cam_pos.x, cam_pos.y, cam_pos.z);

                let view_projection = projection * view;
                let identity = glm::identity::<f32, 4>();
                draw_scene(&scene_root, &simple_shader, &view_projection, &identity);
            }

            // Display the new color buffer on the display
            context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts
        }
    });


    // == //
    // == // From here on down there are only internals.
    // == //


    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events are initially handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::Resized(physical_size), .. } => {
                println!("New window size received: {}x{}", physical_size.width, physical_size.height);
                if let Ok(mut new_size) = arc_window_size.lock() {
                    *new_size = (physical_size.width, physical_size.height, true);
                }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                    input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle Escape and Q keys separately
                match keycode {
                    Escape => { *control_flow = ControlFlow::Exit; }
                    Q      => { *control_flow = ControlFlow::Exit; }
                    _      => { }
                }
            }
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => { }
        }
    });
}
