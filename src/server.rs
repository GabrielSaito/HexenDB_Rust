use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};
use serde::{Deserialize, Serialize};
use crate::database::Database;

#[derive(Serialize, Deserialize)]
enum ServerCommand {
    Connect { db_name: String, encryption_key: String },
    Disconnect,
    Execute { command: String },
    Backup { db_name: String },
    Restore { db_name: String, backup_file: String },
    BeginTransaction,
    CommitTransaction,
    RollbackTransaction,
}

#[derive(Serialize, Deserialize)]
enum ServerResponse {
    Success(String),
    Error(String),
}

pub struct HexenServer {
    databases: Arc<Mutex<HashMap<String, Database>>>,
}

impl HexenServer {
    pub fn new() -> Self {
        HexenServer {
            databases: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start(&self, address: &str) {
        let listener = TcpListener::bind(address).expect("Erro ao iniciar o servidor.");
        println!("Servidor HexenDB iniciado em {}", address);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let databases = self.databases.clone();
                    tokio::spawn(async move {
                        Self::handle_client(stream, databases).await;
                    });
                }
                Err(e) => {
                    eprintln!("Erro ao aceitar conexão: {}", e);
                }
            }
        }
    }

    async fn handle_client(mut stream: TcpStream, databases: Arc<Mutex<HashMap<String, Database>>>) {
        println!("Novo cliente conectado: {:?}", stream.peer_addr().unwrap());

        let mut buffer = [0; 1024];
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => {
                    println!("Cliente desconectado: {:?}", stream.peer_addr().unwrap());
                    break;
                }
                Ok(n) => {
                    let request = String::from_utf8_lossy(&buffer[..n]);
                    let response = Self::process_request(&request, &databases);
                    stream.write_all(response.as_bytes()).unwrap();
                }
                Err(e) => {
                    eprintln!("Erro ao ler do cliente: {}", e);
                    break;
                }
            }
        }
    }

    fn process_request(request: &str, databases: &Arc<Mutex<HashMap<String, Database>>>) -> String {
        let command: Result<ServerCommand, _> = serde_json::from_str(request);

        match command {
            Ok(ServerCommand::Connect { db_name, encryption_key }) => {
                let mut databases = databases.lock().unwrap();
                if databases.contains_key(&db_name) {
                    return serde_json::to_string(&ServerResponse::Error("Banco de dados já está conectado.".to_string()))
                        .unwrap();
                }

                let db = Database::new(format!("{}.hxn", db_name), &encryption_key, "UTF-8");
                databases.insert(db_name, db);
                serde_json::to_string(&ServerResponse::Success("Conexão estabelecida.".to_string())).unwrap()
            }
            Ok(ServerCommand::Disconnect) => {
                serde_json::to_string(&ServerResponse::Success("Desconectado.".to_string())).unwrap()
            }
            Ok(ServerCommand::Execute { command }) => {
                let mut databases = databases.lock().unwrap();
                if let Some(db) = databases.values_mut().next() {
                    return serde_json::to_string(&ServerResponse::Success(db.execute_command(&command))).unwrap();
                }
                serde_json::to_string(&ServerResponse::Error("Nenhum banco de dados conectado.".to_string())).unwrap()
            }
            Ok(ServerCommand::Backup { db_name }) => {
                match Database::create_backup(&db_name, databases) {
                    Ok(message) => serde_json::to_string(&ServerResponse::Success(message)).unwrap(),
                    Err(error) => serde_json::to_string(&ServerResponse::Error(error)).unwrap(),
                }
            }
            Ok(ServerCommand::Restore { db_name, backup_file }) => {
                match Database::restore_backup(&db_name, &backup_file, databases) {
                    Ok(message) => serde_json::to_string(&ServerResponse::Success(message)).unwrap(),
                    Err(error) => serde_json::to_string(&ServerResponse::Error(error)).unwrap(),
                }
            }
            Ok(ServerCommand::BeginTransaction) => {
                let mut databases = databases.lock().unwrap();
                if let Some(db) = databases.values_mut().next() {
                    return serde_json::to_string(&ServerResponse::Success(db.begin_transaction())).unwrap();
                }
                serde_json::to_string(&ServerResponse::Error("Nenhum banco de dados conectado.".to_string())).unwrap()
            }
            Ok(ServerCommand::CommitTransaction) => {
                let mut databases = databases.lock().unwrap();
                if let Some(db) = databases.values_mut().next() {
                    return serde_json::to_string(&ServerResponse::Success(db.commit_transaction())).unwrap();
                }
                serde_json::to_string(&ServerResponse::Error("Nenhum banco de dados conectado.".to_string())).unwrap()
            }
            Ok(ServerCommand::RollbackTransaction) => {
                let mut databases = databases.lock().unwrap();
                if let Some(db) = databases.values_mut().next() {
                    return serde_json::to_string(&ServerResponse::Success(db.rollback_transaction())).unwrap();
                }
                serde_json::to_string(&ServerResponse::Error("Nenhum banco de dados conectado.".to_string())).unwrap()
            }
            Err(_) => serde_json::to_string(&ServerResponse::Error("Comando inválido.".to_string())).unwrap(),
        }
    }
}