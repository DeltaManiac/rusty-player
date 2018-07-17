extern crate rodio;

use std::io::BufReader;

extern crate conrod;

fn main() {
    init();
}

use conrod::backend::glium::glium::{self, Surface};

fn init() {
    let window = glium::glutin::WindowBuilder::new()
        .with_title("rs-player")
        .with_dimensions(320, 640);
    let mut context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let mut event_loop = glium::glutin::EventsLoop::new();
    let display = glium::Display::new(window, context, &event_loop).unwrap();
    let mut event_queue = Vec::new();
    let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();
    let mut ui = conrod::UiBuilder::new([320 as f64, 640 as f64]).build();
    // NOTE(DeltaManiac): Move this to a structperhaps ?
    let device = rodio::default_output_device().unwrap();
    let mut sink = rodio::Sink::new(&device);
    let mut is_playing = false;
    // NOTE(DeltaManiac): Here starts the main loop
    'main: loop {
        event_queue.clear();
        event_loop.poll_events(|event| {
            event_queue.push(event);
        });

        if event_queue.is_empty() {
            event_loop.run_forever(|event| {
                event_queue.push(event);
                glium::glutin::ControlFlow::Break
            });
        }
        for event in event_queue.drain(..) {
            match event.clone() {
                glium::glutin::Event::WindowEvent { event, .. } => match event {
                    glium::glutin::WindowEvent::Closed
                    | glium::glutin::WindowEvent::KeyboardInput {
                        input:
                            glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => break 'main,
                    glium::glutin::WindowEvent::KeyboardInput {
                        input:
                            glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::P),
                                state: glium::glutin::ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        play_music(&mut sink, &mut is_playing);
                        //sink.detach();
                        println!("PRESSEED P");
                        ()
                    }
                    _ => (),
                },
                _ => (),
            };

            let input = match conrod::backend::winit::convert_event(event, &display) {
                None => continue,
                Some(input) => input,
            };

            if let Some(primitives) = ui.draw_if_changed() {
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                target.finish().unwrap();
            }
        }
    }
}

fn play_music(sink: &mut rodio::Sink, is_playing: &mut bool) {
    // TODO(DeltaManiac): Move Audio Code to a better location.
    println!("Called Music");

    if !*is_playing {
        let file = std::fs::File::open("./assets/test.mp3").unwrap();
        if sink.empty() {
            println!("Sink is empty and not playing");
            sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
        } else {
            sink.play();
        }
        *is_playing = true;
    } else {
        sink.pause();
        if sink.empty() {
            println!("Sink is empty");
        }
        *is_playing = false;
    }
}
