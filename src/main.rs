#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate failure;

extern crate config;
extern crate textwrap;
use textwrap::fill;

use rocket_contrib::json::Json;
use rocket::State;

use serde::{Serialize};
use failure::Error;

use std::sync::Mutex;
use std::ops::DerefMut;

mod settings;
use settings::Settings;

mod trello;

struct Context {
    board_id: Option<String>,
    settings: Settings,
    list_index: Mutex<usize>,
}

#[derive(Serialize, Debug)]
struct PyPortal {
    text: String,
    title: String,
    backlight: f32,
}

fn get_backlight_for_light_sensor(light_sensor: u32) -> f32 {
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

            let mut list_index_lock = context.list_index.lock().expect("lock context");
            let list = &lists[*list_index_lock];

            // next request, return the next list
            *list_index_lock.deref_mut() = (*list_index_lock + 1) % lists.len();

            let mut printable_card_list = list.cards.iter().map(|card| format!("* {}", card.name)).collect::<Vec<String>>().join("\n");
            printable_card_list = fill(&printable_card_list, line_len);


            // take only # lines
            printable_card_list = printable_card_list.split("\n").take(lines).collect::<Vec<&str>>().join("\n");

            Json(PyPortal {
                text: printable_card_list,
                title: list.name.clone(),
                backlight: get_backlight_for_light_sensor(light_sensor),
            })
        },
        None => Json(PyPortal {
            text: String::from("couldn't get board id"),
            title: String::from("error"),
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
        list_index: Mutex::new(0),
    };

    println!("Hello, world!");
    rocket::ignite()
        .manage(context)
        .mount("/", routes![index])
        .launch();
}
