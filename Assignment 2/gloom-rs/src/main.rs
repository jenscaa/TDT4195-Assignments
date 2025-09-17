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

use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;

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


// ##############################################################################
// TASK 1a) 

// Creates a VAO and returns its id
unsafe fn create_vao(vertices: &Vec<f32>, indices: &Vec<u32>, colors: &Vec<f32>) -> u32 {
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

    return vao_id;
}
// ##############################################################################


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

        // ###############################################

        // Triangles for task 1 b)

        // let vertices: Vec<f32> = vec![

        //     // Triangle 1
        //     -0.3,  0.4, 0.0,
        //     0.3,  0.4, 0.0,
        //     0.0,  0.8, 0.0,

        //     // Triangle 2
        //     -0.85,  0.55, 0.0,
        //     -0.65,  0.55, 0.0,
        //     -0.75,  0.80, 0.0,

        //     // Triangle 3
        //     0.69,  0.59, 0.0,
        //     0.89,  0.59, 0.0,
        //     0.75,  0.80, 0.0,

        //     // Triangle 4
        //     -0.95, -0.05, 0.0,
        //     -0.55, -0.05, 0.0,
        //     -0.75, 0.35, 0.0,

        //     // Trinagle 5
        //     -0.21, -0.05, 0.0,
        //     0.21, -0.05, 0.0,
        //     0.0, 0.2, 0.0,
        // ];

        // let indices: Vec<u32> = vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14];

        // let colors: Vec<f32> = vec![

        //     // Triangle 1 (red, green, blue)
        //     1.0, 0.0, 0.0, 1.0,
        //     0.0, 1.0, 0.0, 1.0,
        //     0.0, 0.0, 1.0, 1.0,

        //     // Triangle 2 (yellow, magenta, cyan)
        //     1.0, 1.0, 0.0, 1.0,
        //     1.0, 0.0, 1.0, 1.0,
        //     0.0, 1.0, 1.0, 1.0,

        //     // Triangle 3 (orange, white, black)
        //     1.0, 0.5, 0.0, 1.0,
        //     1.0, 1.0, 1.0, 1.0,
        //     0.0, 0.0, 0.0, 1.0,

        //     // Triangle 4 (gray shades)
        //     0.2, 0.2, 0.2, 1.0,
        //     0.5, 0.5, 0.5, 1.0,
        //     0.8, 0.8, 0.8, 1.0,

        //     // Triangle 5 (transparent red to solid red)
        //     1.0, 0.0, 0.0, 0.2,
        //     1.0, 0.0, 0.0, 0.6,
        //     1.0, 0.0, 0.0, 1.0,
        // ];

        // let vao = unsafe { create_vao(&vertices, &indices, &colors) };
        // let index_count = indices.len() as i32;

        // ###############################################

        // Triangles for task 2 a)

        let vertices: Vec<f32> = vec![
            // Triangle 1 (furthest) – largest triangle
            -0.6, -0.4, 0.1,
            0.6, -0.4, 0.1,
            0.0,  0.6, 0.1,

            // Triangle 2 (middle) – medium triangle
            -0.45, -0.45, 0.2,
            0.45, -0.45, 0.2,
            0.0,   0.45, 0.2,

            // Triangle 3 (closest) – smallest triangle
            -0.3, -0.5, 0.3,
            0.3, -0.5, 0.3,
            0.0,  0.3, 0.3,
        ];


        let indices: Vec<u32> = vec![
            0, 1, 2,  // Triangle 1
            3, 4, 5,  // Triangle 2
            6, 7, 8,  // Triangle 3
        ];

            let colors: Vec<f32> = vec![
            // Triangle 1 (furthest) – Light Yellow
            0.98, 0.95, 0.70, 0.6,
            0.98, 0.95, 0.70, 0.6,
            0.98, 0.95, 0.70, 0.6,

            // Triangle 2 (middle) – Light Red
            0.96, 0.72, 0.72, 0.6,
            0.96, 0.72, 0.72, 0.6,
            0.96, 0.72, 0.72, 0.6,

            // Triangle 3 (closest) – Light Blue
            0.68, 0.85, 0.90, 0.6,
            0.68, 0.85, 0.90, 0.6,
            0.68, 0.85, 0.90, 0.6,
        ];

        let vao = unsafe { create_vao(&vertices, &indices, &colors) };
        let index_count = indices.len() as i32;
        // ###############################################


        // Used to demonstrate keyboard handling for exercise 2.
        let mut _arbitrary_number = 0.0; // feel free to remove


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

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        // The `VirtualKeyCode` enum is defined here:
                        //    https://docs.rs/winit/0.25.0/winit/event/enum.VirtualKeyCode.html

                        VirtualKeyCode::A => {
                            _arbitrary_number += delta_time;
                        }
                        VirtualKeyCode::D => {
                            _arbitrary_number -= delta_time;
                        }


                        // default handler:
                        _ => { }
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {

                // == // Optionally access the accumulated mouse movement between
                // == // frames here with `delta.0` and `delta.1`

                *delta = (0.0, 0.0); // reset when done
            }

            // == // Please compute camera transforms here (exercise 2 & 3)


            unsafe {
                // Clear the color and depth buffers
                gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                
                // For drawing traingles
                simple_shader.activate(); 
                gl::BindVertexArray(vao);
                gl::DrawElements(
                    gl::TRIANGLES,
                    index_count,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                );
                gl::BindVertexArray(0);



            }

            // Display the new color buffer on the display
            context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts

            

            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let val = elapsed.sin();

            unsafe {
                let cname = std::ffi::CString::new("u_val").unwrap();
                let loc = gl::GetUniformLocation(simple_shader.program_id, cname.as_ptr());
                gl::Uniform1f(loc, val);
            }
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
