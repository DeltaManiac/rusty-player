
extern crate button_controller;
extern crate music;
extern crate piston_window;
use button_controller::{ButtonController, ButtonEvent, ButtonState};
use piston_window::*;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
enum Music {
    s1,
}
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
enum Sound {
    s1,
}

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Test music", [640, 480])
        .vsync(true)
        .controllers(false)
        .decorated(false)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut quit = ButtonController::new();
    let quit_pos = [0.0, 0.0, 20.0, 20.0];

    let mut play_button = ButtonController::new();
    let play_button_pos = [120.0, 120.0, 50.0, 50.0];
    let mut is_playing = false;

    music::start::<Music, Sound, _>(16, || {
        music::bind_music_file(Music::s1, "./assets/test.mp3");
        music::play_music(&Music::s1, music::Repeat::Forever);

        music::set_volume(music::MIN_VOLUME);

        'outer: while let Some(e) = window.next() {
            quit.event(quit_pos, math::identity(), &e);
            for e in &quit.events {
                match e {
                    ButtonEvent::Click => {
                        println!("Clicked");
                        break 'outer;
                    }
                    _ => (),
                }
            }

            play_button.event(play_button_pos, math::identity(), &e);
            for e in &play_button.events {
                match e {
                    ButtonEvent::Click => {
                        println!("Current Playing status : {}", is_playing);
                        
                        if is_playing {
                                music::play_music(&Music::s1, music::Repeat::Forever);
                            
                                music::set_volume(music::MAX_VOLUME);
                        } else {
                            music::set_volume(music::MIN_VOLUME);
                        }
                        is_playing = !is_playing;
                    }
                    
                    _ => (),
                };
                
            }

            window.draw_2d(&e, |_c, g| {
                clear([0.0; 4], g);

                let quit_color = match quit.state(0.2) {
                    ButtonState::Hover => [1.0, 0.0, 0.0, 1.0],
                    ButtonState::Press => [0.1, 0.1, 0.1, 1.0],
                    //ButtonState::Inactive => [0.4, 0.4, 0.4, 1.0],
                    _ => [0.4, 0.4, 0.4, 1.0],
                    //ButtonState::Cancel => [0.3, 0.2, 0.2, 1.0],
                };
                rectangle(quit_color, quit_pos, _c.transform, g);

                let play_button_col = if is_playing {
                    [0.0, 1.0, 0.0, 1.0]
                } else {
                    [0.0, 1.0, 1.0, 1.0]
                };

                rectangle(play_button_col, play_button_pos, _c.transform, g);
            });
        }
    });
}
