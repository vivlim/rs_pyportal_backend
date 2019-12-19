#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate failure;

extern crate config;

use rocket_contrib::json::Json;
use rocket::State;

use serde::{Serialize};
use failure::Error;

mod settings;
use settings::Settings;

mod trello;

struct Context {
    board_id: Option<String>,
    settings: Settings
}

#[derive(Serialize, Debug)]
struct PyPortal {
    text: String,
    author: String,
}

#[get("/")]
fn index(context: State<Context>) -> Json<PyPortal> {
    match &context.board_id {
        Some(id) => {
            let lists = trello::get_lists_for_board(&id, &context.settings).unwrap();
            Json(PyPortal {
                text: lists[0].cards[0].name.clone(),
                author: String::from("whatever")
            })
        },
        None => Json(PyPortal {
            text: String::from("couldn't get board id"),
            author: String::from("error")
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
        settings: settings
    };

    println!("Hello, world!");
    rocket::ignite()
        .manage(context)
        .mount("/", routes![index])
        .launch();
}
