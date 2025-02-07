mod client;
mod database;
mod server;
mod table;
mod user_manager;

use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Uso: cargo run -- <server|client> [argumentos]");
        return;
    }

    match args[1].as_str() {
        "server" => {
            let server = server::HexenServer::new();
            server.start("127.0.0.1:8060").await;
        }
        "client" => {
            let mut client = client::HexenClient::new("127.0.0.1:8060");
            client::run_client(client);
        }
        _ => println!("Modo inválido. Use 'server' ou 'client'."),
    }
}
