extern crate reqwest;
extern crate ring;
extern crate data_encoding;

#[macro_use] extern crate prettytable;
use prettytable::{Table};

use std::io;

mod binance;
use binance::api::Order;

mod app_settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_settings =  app_settings::load_or_create_settings();

    let binance_api = binance::api::new(
        app_settings.binance_key.to_string(),
        app_settings.binance_secret_key.to_string()
    );
    let user_orders = binance_api.get_open_orders().await;
    print_as_table(&user_orders);
    println!("");
    println!("Хотите закрыть все сделки? (y\\n)");
    let mut user_input = String::new();
    loop {
        let _r = io::stdin().read_line(&mut user_input).unwrap();
        user_input = user_input.trim().to_string();
        if user_input == "y" || user_input == "n" {
            break;
        }
        user_input = String::new();
    }

    if user_input == "y" {
        for order in user_orders {
            binance_api.close_all_orders(&order.symbol).await;
        }
        println!("Все сделки закрыты");
    }

    println!("Нажмите ENTER, чтобы выйти");
    let _r = io::stdin().read_line(&mut user_input).unwrap();

    Ok(())
}


fn print_as_table(orders: &Vec<Order>) {
    let mut table = Table::new();
    
    table.add_row(row!["Symbol", "Side", "Price", "Quantity", "Type"]);
    for order in orders {
        table.add_row(row![
            order.symbol, 
            order.side, 
            order.price, 
            order.orig_qty, 
            order.orig_type
        ]);
    }
    table.printstd();
}