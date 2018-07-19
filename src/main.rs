extern crate rodio;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::borrow::Cow;
#[macro_use]
extern crate conrod;

fn main() {
    init();
}

use conrod::{
    backend::glium::glium::{self, Surface},
    widget, Colorable, Positionable, Widget,
};


struct current_mp3<'a> {
idx:u32,
    id3v1_tag : Id3V1<'a>
}
impl<'a> current_mp3<'a> {
        pub fn new(file:Option<&mut File>) -> current_mp3<'a> {
            let tag = match file {
                Some(file)=> Id3V1::from_file(file),
                None => Id3V1::new()
        };
            current_mp3 {
                idx:0,
            id3v1_tag:tag,
        }
    }
 }


fn init() {
    let (width, height) = (640, 320);
    let window = glium::glutin::WindowBuilder::new()
        .with_title("rs-player")
        .with_min_dimensions(width,height)
        // TODO(DeltaManiac): Find out why this doesnt work
        //.with_resizable(false)
        .with_decorations(false);
    let mut context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let mut event_loop = glium::glutin::EventsLoop::new();
    let display = glium::Display::new(window, context, &event_loop).unwrap();
    let mut event_queue = Vec::new();
    let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();
    let mut ui = conrod::UiBuilder::new([width as f64, height as f64]).build();
    // NOTE(DeltaManiac): Recheck this
    widget_ids!(struct Ids { 
            text,
        text_info,
            text_title,
    text_artist,
    text_album,
        text_year,
    }
    );
        let mut curr_file = current_mp3::new(None);
    let ids = Ids::new(ui.widget_id_generator());
    ui.fonts.insert_from_file("./assets/Potra.ttf").unwrap();
    let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();
    // NOTE(DeltaManiac): Move this to a struct perhaps ?
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
                        curr_file = play_music(&mut sink, &mut is_playing);
                            //println!("PRESSEED P:{:?}",curr_file);
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
                             ui.handle_event(input);
                             let ui = &mut ui.set_widgets();
                             if is_playing {
                             widget::Text::new("Press P to Pause!")
                             .middle_of(ui.window)
                             .color(conrod::color::LIGHT_YELLOW)
                             .font_size(28)
                             .set(ids.text, ui);
                             
                             widget::Text::new("File Info") 
                             .top_left_of(ui.window)
                             .color(conrod::color::LIGHT_RED)
                             .font_size(24)
                             .set(ids.text_info, ui);
                             
                             widget::Text::new(&format!("Title:{}" , curr_file.id3v1_tag.title.to_owned())[..]) 
                             .down_from(ids.text_info,1.0)
                             .color(conrod::color::LIGHT_BLUE)
                             .font_size(20)
                             .set(ids.text_title, ui);
                             
                             widget::Text::new(&format!("Album:{}" , curr_file.id3v1_tag.album.to_owned())[..]) 
                             .down_from(ids.text_title,1.0)
                             .color(conrod::color::LIGHT_BLUE)
                             .font_size(20)
                             .set(ids.text_album, ui);
                             
                             widget::Text::new(&format!("Artist:{}" , curr_file.id3v1_tag.artist.to_owned())[..]) 
                             .down_from(ids.text_album,1.0)
                             .color(conrod::color::LIGHT_BLUE)
                             .font_size(20)
                             .set(ids.text_artist, ui);
                             
                             widget::Text::new(&format!("Year:{}" , curr_file.id3v1_tag.year.to_owned())[..]) 
                             .down_from(ids.text_artist,1.0)
                             .color(conrod::color::LIGHT_BLUE)
                             .font_size(20)
                             .set(ids.text_year, ui);
                             
                             
                             } else {
                             widget::Text::new("Press P to Play!")
                             .middle_of(ui.window)
                             .color(conrod::color::LIGHT_YELLOW)
                             .font_size(28)
                             .set(ids.text, ui);
                             }
                             //println!("{:?}",a);
            if let Some(primitives) = ui.draw_if_changed() {
                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }
        }
    }
}
/*
fn play_music(sink: &mut rodio::Sink, is_playing: &mut bool) {
    // TODO(DeltaManiac): Move Audio Code to a better location.
    println!("Called Music");

    if !*is_playing {
        //let mut file = std::fs::File::open("./assets/test.mp3").unwrap();
            let mut file = std::fs::File::open("./assets/def.mp3").unwrap();
        print_mp3_tag(&mut file);
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

*/
fn play_music<'a>(sink: &mut rodio::Sink, is_playing: &mut bool) -> current_mp3<'a> {
        // TODO(DeltaManiac): Move Audio Code to a better location.
        println!("Called Music");
    
        if !*is_playing {
            let mut file = std::fs::File::open("./assets/test.mp3").unwrap();
            //let mut file = std::fs::File::open("./assets/def.mp3").unwrap();
            let tag = current_mp3::new(Some(&mut file));
            print_mp3_tag(&mut file);
            if sink.empty() {
                println!("Sink is empty and not playing");
                sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
        } else {
            sink.play();
        }
        *is_playing = true;
        tag
    } else {
        sink.pause();
            if sink.empty() {
                println!("Sink is empty");
        }
        *is_playing = false;
            current_mp3::new(None)
    }
    
}

#[derive(Debug)]
struct Id3V1<'a> {
    title: Cow<'a, str>,
    artist: Cow<'a, str>,
        year: Cow<'a, str>,
        album:  Cow<'a, str>,
}

impl<'a> Id3V1<'a> {
    pub fn new() -> Id3V1<'a> {
        Id3V1 {
                title: Cow::Borrowed("Untitled"),
                artist: Cow::Borrowed("Unknown"),
year: Cow::Borrowed("Unknown"),
                album: Cow::Borrowed("Unknown"),
        }
    }

    pub fn from_file(file: &mut File) -> Id3V1<'a> {
        println!("Seek end -128{:?}", file.seek(SeekFrom::End(-128)));
        let mut tag_data = Id3V1::new();
        let mut data: Vec<u8> = Vec::new();
        file.read_to_end(&mut data);
        // NOTE(DeltaManiac): TAG == TAG
        if (data[0], data[1], data[2]) == (84, 65, 71) {
            
                tag_data.title = Cow::Owned(std::str::from_utf8(&data[3..33])
                .unwrap()
                .trim_matches(|char| char == '\0').to_string());
                tag_data.artist = Cow::Owned(std::str::from_utf8(&data[33..63])
                .unwrap()
                .trim_matches(|char| char == '\0').to_string());
                tag_data.album = Cow::Owned(std::str::from_utf8(&data[63..93])
                .unwrap()
                .trim_matches(|char| char == '\0').to_string());
                tag_data.year = Cow::Owned(std::str::from_utf8(&data[93..97])
                .unwrap()
                .trim_matches(|char| char == '\0').to_string());
        }
        file.seek(SeekFrom::Start(0));
        tag_data
    }
}

fn print_mp3_tag(file: &mut File) {
    println!("{:?}", Id3V1::from_file(file));
    // NOTE(DeltaManiac): Reset if cursor has moved
    //file.seek(SeekFrom::Start(0));
}
