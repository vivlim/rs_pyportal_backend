use reqwest;
use serde::{Deserialize};
use failure::Error;

use crate::settings::Settings;

#[derive(Deserialize, Debug, Clone)]
pub struct TrelloBoard {
    pub name: String,
    pub id: String,
}

#[derive(Deserialize, Debug)]
pub struct TrelloList {
    pub id: String,
    pub name: String,
    pub cards: Vec<TrelloCard>,
}

#[derive(Deserialize, Debug)]
pub struct TrelloCard {
    pub id: String,
    pub name: String,
    pub closed: bool,
    pub url: String
}

pub fn get_board_by_name(board_name: &str, settings: &Settings) -> Result<TrelloBoard, Error>{
    let get_boards_uri = format!("https://api.trello.com/1/members/me/boards?fields=name&key={key}&token={token}", key = settings.trello_key, token = settings.trello_token);
    println!("get boards uri: {}", get_boards_uri);
    let resp: Vec<TrelloBoard> = reqwest::get(&get_boards_uri)?.json()?;

    match resp.iter().find(|item| item.name == board_name) {
        Some(trello_board) => Ok(trello_board.clone()),
        None => Err(format_err!("Board not found."))
    }
}

pub fn get_lists_for_board(board_id: &String, settings: &Settings) -> Result<Vec<TrelloList>, Error>{
    let get_lists_uri = format!("https://api.trello.com/1/boards/{boardid}/lists?cards=open&key={key}&token={token}", boardid = board_id, key = settings.trello_key, token = settings.trello_token);
    println!("get lists uri: {}", get_lists_uri);
    let lists: Vec<TrelloList> = reqwest::get(&get_lists_uri)?.json()?;
    println!("{}", lists[0].cards[0].name);
    Ok(lists)
}