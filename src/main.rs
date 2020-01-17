#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate failure;

extern crate config;
extern crate chrono;
use chrono::{Local};
extern crate textwrap;
use textwrap::fill;

use rocket_contrib::json::Json;
use rocket::State;

use serde::{Serialize};
use failure::Error;

use std::sync::Mutex;
use std::ops::DerefMut;
use std::fs::OpenOptions;
use std::io::prelude::*;

mod settings;
use settings::Settings;

mod trello;

struct Context {
    board_id: Option<String>,
    settings: Settings,
    card_index: Mutex<usize>,
}

#[derive(Serialize, Debug)]
struct PyPortal {
    text: String,
    backlight: f32,
}

fn get_backlight_for_light_sensor(light_sensor: u32) -> f32 {
    let mut file = OpenOptions::new()
        .append(true)
        .open("light_sensor.csv")
        .unwrap();

    if let Err(e) = writeln!(file, "{},{}", Local::now(), light_sensor) {
        eprintln!("Couldn't write sensor data to file {}", e);
    }

    if light_sensor > 700 {
        return 0.5;
    }
    return 0.0;
}

#[get("/?<lines>&<line_len>&<light_sensor>")]
fn index(lines: usize, line_len: usize, light_sensor: u32, context: State<Context>) -> Json<PyPortal> {
    match &context.board_id {
        Some(id) => {
            let lists = trello::get_lists_for_board(&id, &context.settings).unwrap();
            let list = &lists[1];

            let mut card_index_lock = context.card_index.lock().expect("lock context");
            let card = &list.cards[*card_index_lock];

            // next request, return the next list
            *card_index_lock.deref_mut() = (*card_index_lock + 1) % list.cards.len();

            let mut printable_card = fill(&card.name, line_len);

            // take only # lines
            printable_card = printable_card.split("\n").take(lines).collect::<Vec<&str>>().join("\n");

            Json(PyPortal {
                text: printable_card,
                backlight: get_backlight_for_light_sensor(light_sensor),
            })
        },
        None => Json(PyPortal {
            text: String::from("couldn't get board id"),
            backlight: get_backlight_for_light_sensor(light_sensor),
        })
    }
}

fn main() {
    let settings = Settings::new().unwrap();

    let context = Context {
        board_id: match trello::get_board_by_name("Shared Tasks (V&R)", &settings) {
            Ok(board) => Some(board.id),
            Err(e) => {
                    println!("Couldn't get the board id. {:?}", e);
                    None
                }
        },
        settings: settings,
        card_index: Mutex::new(0),
    };

    println!("Hello, world!");
    rocket::ignite()
        .manage(context)
        .mount("/", routes![index])
        .launch();
}
