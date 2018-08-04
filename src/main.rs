extern crate rodio;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;
use std::thread;
#[macro_use]
extern crate conrod;

use conrod::{
    backend::glium::glium::{self, Surface},
    widget, Colorable, Positionable, Sizeable, Widget,
};

fn main() {
    //init();
    //Better Window
    init_2();
    println!("DONE");
}

//------------------------------------STRUCTS----------------------------------\\

#[derive(Debug, Eq)]
struct Id3V1<'a> {
    title: Cow<'a, str>,
    artist: Cow<'a, str>,
    year: Cow<'a, str>,
    album: Cow<'a, str>,
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
            tag_data.title = Cow::Owned(
                std::str::from_utf8(&data[3..33])
                    .unwrap()
                    .trim_matches(|char| char == '\0')
                    .to_string(),
            );
            tag_data.artist = Cow::Owned(
                std::str::from_utf8(&data[33..63])
                    .unwrap()
                    .trim_matches(|char| char == '\0')
                    .to_string(),
            );
            tag_data.album = Cow::Owned(
                std::str::from_utf8(&data[63..93])
                    .unwrap()
                    .trim_matches(|char| char == '\0')
                    .to_string(),
            );
            tag_data.year = Cow::Owned(
                std::str::from_utf8(&data[93..97])
                    .unwrap()
                    .trim_matches(|char| char == '\0')
                    .to_string(),
            );
        }
        file.seek(SeekFrom::Start(0));
        tag_data
    }
}

impl<'a> Default for Id3V1<'a> {
    fn default() -> Id3V1<'a> {
        Id3V1 {
            title: Cow::Borrowed("Untitled"),
            artist: Cow::Borrowed("Unknown"),
            year: Cow::Borrowed("Unknown"),
            album: Cow::Borrowed("Unknown"),
        }
    }
}

impl<'a> PartialEq for Id3V1<'a> {
    fn eq(&self, other: &Id3V1) -> bool {
        self.title == other.title
            && self.artist == other.artist
            && self.album == other.album
            && self.year == other.year
    }
}

#[derive(Default)]
struct PlayListItem<'a> {
    file_name: Cow<'a, str>,
    file_path: Cow<'a, std::path::PathBuf>,
    sink: Option<Box<rodio::Sink>>,
        playing: bool,
    id3v1: Box<Id3V1<'a>>,
}

impl <'a> PlayListItem<'a> {
    
        fn init_sink( mut self) {
            let device = rodio::default_output_device().unwrap();
            let mut sink = rodio::Sink::new(&device);
            self.sink= Some(Box::new(sink));
            
    }

    fn play_item(mut self) {
            let mut file = std::fs::File::open(self.file_path.to_str().unwrap()).unwrap();
            self.sink.unwrap().append(rodio::Decoder::new(BufReader::new(file)).unwrap());
            self.playing = true;
    }
}


impl<'a> Ord for PlayListItem<'a> {
    fn cmp(&self, other: &PlayListItem) -> Ordering {
        self.file_name.cmp(&other.file_name)
    }
}

impl<'a> PartialOrd for PlayListItem<'a> {
    fn partial_cmp(&self, other: &PlayListItem) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> PartialEq for PlayListItem<'a> {
    fn eq(&self, other: &PlayListItem) -> bool {
        self.file_name == other.file_name
    }
}

impl<'a> Eq for PlayListItem<'a> {}

impl<'a> fmt::Debug for PlayListItem<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Test")
    }
}

#[derive(Debug)]
struct current_mp3<'a> {
    idx: u32,
    id3v1_tag: Id3V1<'a>,
}

impl<'a> current_mp3<'a> {
    pub fn new(file: Option<&mut File>) -> current_mp3<'a> {
        let tag = match file {
            Some(file) => Id3V1::from_file(file),
            None => Id3V1::new(),
        };
        current_mp3 {
            idx: 0,
            id3v1_tag: tag,
        }
    }
}

//--------------------------------------II-------------------------------------\\

fn init_2() {
    let (width, height) = (800, 600);
    let window = glium::glutin::WindowBuilder::new()
        .with_title("rs-player")
        .with_dimensions(width, height);
    // TODO(DeltaManiac): Find out why this doesnt work
    //.with_resizable(false)
    //.with_decorations(false);
    let mut context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let mut event_loop = glium::glutin::EventsLoop::new();
    let mut event_queue = Vec::new();
    let display = glium::Display::new(window, context, &event_loop).unwrap();
    //let mut event_queue = Vec::new();
    let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();
    let mut ui = conrod::UiBuilder::new([width as f64, height as f64]).build();
    let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();
    //let mut list: Vec<String> = Vec::new();
    let mut list: Vec<PlayListItem> = Vec::new();
    let mut is_adding_file = false;
    let device = rodio::default_output_device().unwrap();
    let mut sink = rodio::Sink::new(&device);

    widget_ids! {
            struct Ids {
                master,
                play_bar,
                play_area,
                play_list,
                play_file_navigator,
                dummy,
        }
    }
    let mut ids = Ids::new(ui.widget_id_generator());
    ui.fonts
        .insert_from_file("./assets/liberation-mono.ttf")
        .unwrap();
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
                    _ => (),
                }, // match event
                _ => (),
            } //event.clone

            let input = match conrod::backend::winit::convert_event(event, &display) {
                None => continue,
                Some(input) => input,
            };
            ui.handle_event(input);
        }

        init_ui(
            ui.set_widgets(),
            &mut ids,
            &mut list,
            &mut is_adding_file,
            &mut sink,
        );
        if let Some(primitives) = ui.draw_if_changed() {
            renderer.fill(&display, primitives, &image_map);
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
    } //event_queue.drain

    /*
    Create the UI as follows
      ----------------------
      |    |              /|
    |    |               |
    |play|    ???        |
    |list|               |
    |    |               |
    |    |               |
    ----------------------
    |       playbar      |
    ----------------------
    */
    fn init_ui(
        ref mut ui: conrod::UiCell,
        ids: &mut Ids,
        list: &mut Vec<PlayListItem>,
        is_adding_file: &mut bool,
        sink: &mut rodio::Sink,
    ) {
        use conrod::{color, widget, Colorable, Labelable, Positionable, Sizeable, Widget};

        widget::Canvas::new()
            .flow_down(&[
                (
                    ids.play_area,
                    widget::Canvas::new()
                        .color(color::BLUE)
                        .scroll_kids_vertically()
                        .length_weight(75.0),
                ),
                (
                    ids.play_bar,
                    widget::Canvas::new()
                        .color(color::RED)
                        .length_weight(25.0)
                        .scroll_kids_vertically(),
                ),
            ])
            .set(ids.master, ui);

        for _click in widget::Button::new()
            .middle_of(ids.play_bar)
            .w_h(80.0, 80.0)
            .label("Open/Close")
            .set(ids.dummy, ui)
        {
            //println!("Before:{:?}", is_adding_file);
            if *is_adding_file {
                *is_adding_file = false;
            } else {
                *is_adding_file = true;
            }
            //println!("After:{:?}", is_adding_file);
        }

        if *is_adding_file {
            //We Open the file dialog
            for _event in widget::FileNavigator::with_extension(&Path::new("."), &["mp3"])
                .color(conrod::color::BLUE)
                .text_color(conrod::color::GREEN)
                .unselected_color(conrod::color::BLACK)
                .font_size(16)
                .wh_of(ui.window)
                //.top_left_of(ui.window)
                .top_left_of(ids.play_area)
                //.show_hidden_files(true)  // Use this to show hidden files
                .set(ids.play_file_navigator, ui)
            {
                // TODO(DeltaManiac): Fix the detection of mp3
                // TODO(DeltaManiac): Find why PathBuf.ends_with fails
                match _event {
                    widget::file_navigator::Event::DoubleClick(_, items) => {
                        println!("Selected Items : {:?} ", items);
                        let files: Vec<std::path::PathBuf> = items
                            .into_iter()
                            .take_while(|p| p.is_file() && p.to_str().unwrap().ends_with(".mp3"))
                            .collect();
                        println!("Items to be added : {:?} ", files);
                        for file in files {
                            list.push(PlayListItem {
                                file_name: Cow::Owned(
                                    file.file_name().unwrap().to_str().unwrap().to_string(),
                                ),
                                file_path: Cow::Owned(
                                    file.canonicalize().unwrap(), //.to_str().unwrap().to_string(),
                                ),
                                playing: false,
                                sink: None,
                                id3v1: Box::new(Default::default()),
                            });
                        }
                        list.sort();
                        list.dedup();
                        println!("list {:?}", list);
                        *is_adding_file = false;
                        ()
                    }
                    _ => (),
                } //End of Match
            }
        } else {
            //We show the current list
            let (mut events, scrollbar) =
                widget::ListSelect::new(list.len(), conrod::widget::list_select::Multiple)
                    .flow_down()
                    .item_size(20.0)
                    .scrollbar_next_to()
                    .instantiate_all_items()
                    .w(200.0)
                    .h_of(ids.play_area)
                    .top_left_of(ids.play_area)
                    .set(ids.play_list, ui);

            if let Some(s) = scrollbar {
                s.set(ui);
            }

            let mut is_selected = false;
            //FOR LIST SELECT
            while let Some(event) = events.next(ui, |i| true) {
                use conrod::widget::list_select::Event;
                match event {
                    Event::Item(item) => {
                        let idx = item.i;
                             let ctrl = widget::Button::new()
                             //.middle_of(ids.play_bar)
                             .w_h(80.0, 80.0)
                             .label(&list[idx].file_name);
                             let times_clicked = item.set(ctrl, ui);
                             for _click in times_clicked
                             {
                             //println!("{:?}",&list[idx].file_path.to_owned());
                             let mut file = std::fs::File::open((&list[idx]).file_path.to_str().unwrap() ).unwrap();
                             println!("{:?}", Id3V1::from_file(&mut file));
                             //let device = rodio::default_output_device().unwrap();
                             //let mut sink = rodio::Sink::new(&device);
                             println!("{:?}",file.metadata());
                             /*if sink.empty() {
                             println!("Sink is empty and not playing");
                             sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
                             //sink.sleep_until_end();
                             sink.stop();
                             } else {
                             sink.play();
                             }
                             */
                             &list[idx].init_sink();
                             &list[idx].play_item();
                             println!("PLAY THE SONG");
                             }
                             ()
                    }
                              event => ()//println!("anything ekse {:?}", &event),
                } // End of Match
            } //End of while
        }
    }
}

//----------------------------------------------I-------------------------------------------\\

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
                              file_navigator,
                              }
                              );

    ////////////////////////////////////////////
    // NOTE(DeltaManiac): Fix this
    let mut curr_file_path = "./assets/test.mp3";
    let mut curr_file = current_mp3::new(None);
    // NOTE(DeltaManiac): Move this to a struct perhaps ?
    let device = rodio::default_output_device().unwrap();
    let mut sink = rodio::Sink::new(&device);
    let mut is_playing = false;
    let mut is_opening = false;
    /////////////////////////////////////////////
    let ids = Ids::new(ui.widget_id_generator());
    ui.fonts.insert_from_file("./assets/Potra.ttf").unwrap();
    let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

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
                        curr_file = play_music(&mut sink, &mut is_playing, curr_file_path);
                        //println!("PRESSEED P:{:?}",curr_file);
                        ()
                    }
                    glium::glutin::WindowEvent::KeyboardInput {
                        input:
                            glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::O),
                                state: glium::glutin::ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        if !is_opening {
                            is_opening = true;
                        } else {
                            is_opening = false;
                        }
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

                widget::Text::new(&format!("Title:{}", curr_file.id3v1_tag.title.to_owned())[..])
                    .down_from(ids.text_info, 1.0)
                    .color(conrod::color::LIGHT_BLUE)
                    .font_size(20)
                    .set(ids.text_title, ui);

                widget::Text::new(&format!("Album:{}", curr_file.id3v1_tag.album.to_owned())[..])
                    .down_from(ids.text_title, 1.0)
                    .color(conrod::color::LIGHT_BLUE)
                    .font_size(20)
                    .set(ids.text_album, ui);

                widget::Text::new(&format!("Artist:{}", curr_file.id3v1_tag.artist.to_owned())[..])
                    .down_from(ids.text_album, 1.0)
                    .color(conrod::color::LIGHT_BLUE)
                    .font_size(20)
                    .set(ids.text_artist, ui);

                widget::Text::new(&format!("Year:{}", curr_file.id3v1_tag.year.to_owned())[..])
                    .down_from(ids.text_artist, 1.0)
                    .color(conrod::color::LIGHT_BLUE)
                    .font_size(20)
                    .set(ids.text_year, ui);
            } else {
                widget::Text::new("Press P to Play!\nPress O to Open/Close Files!")
                    .middle_of(ui.window)
                    .color(conrod::color::LIGHT_YELLOW)
                    .font_size(28)
                    .set(ids.text, ui);
                if is_opening {
                    for _event in widget::FileNavigator::with_extension(&Path::new("."), &["mp3"])
                               .color(conrod::color::BLUE)
                               .text_color(conrod::color::GREEN)
                               .unselected_color(conrod::color::BLACK)
                               .font_size(16)
                               .wh_of(ui.window)
                               .top_left_of(ui.window)
                               //.show_hidden_files(true)  // Use this to show hidden files
                               .set(ids.file_navigator, ui)
                    {
                        //println!("{:?}", _event);
                        // TODO(DeltaManiac): Better event handling
                        let mut a: Vec<std::path::PathBuf> = Vec::new();
                        match _event {
                            widget::file_navigator::Event::DoubleClick(_, a) => {
                                println!("{:?}", a);
                                //curr_file_path = &a[0].to_str().unwrap();
                                ()
                            }
                            _ => (),
                        }
                    }
                }
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

fn play_music<'a>(sink: &mut rodio::Sink, is_playing: &mut bool, path: &str) -> current_mp3<'a> {
    // TODO(DeltaManiac): Move Audio Code to a better location.
    println!("Called Music");

    if !*is_playing {
        let mut file = std::fs::File::open(path).unwrap();
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

fn print_mp3_tag(file: &mut File) {
    println!("{:?}", Id3V1::from_file(file));
    // NOTE(DeltaManiac): Reset if cursor has moved
    //file.seek(SeekFrom::Start(0));
}
