use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::net::TcpStream;

#[derive(Serialize, Deserialize)]
enum ClientCommand {
    Connect {
        db_name: String,
        encryption_key: String,
    },
    Disconnect,
    Execute {
        command: String,
    },
    Backup {
        db_name: String,
    },
    Restore {
        db_name: String,
        backup_file: String,
    },
    BeginTransaction,
    CommitTransaction,
    RollbackTransaction,
}

#[derive(Serialize, Deserialize)]
enum ServerResponse {
    Success(String),
    Error(String),
}

pub struct HexenClient {
    stream: TcpStream,
}

impl HexenClient {
    pub fn new(address: &str) -> Self {
        let stream = TcpStream::connect(address).expect("Erro ao conectar ao servidor.");
        HexenClient { stream }
    }

    pub fn send_command(&mut self, command: ClientCommand) -> String {
        let request = serde_json::to_string(&command).unwrap();
        self.stream.write_all(request.as_bytes()).unwrap();

        let mut buffer = [0; 1024];
        let n = self.stream.read(&mut buffer).unwrap();
        String::from_utf8_lossy(&buffer[..n]).to_string()
    }
}

pub fn run_client(mut client: HexenClient) {
    loop {
        println!("\nEscolha uma opção:");
        println!("1. Conectar ao banco de dados");
        println!("2. Executar comando SQL");
        println!("3. Criar backup");
        println!("4. Restaurar backup");
        println!("5. Iniciar transação");
        println!("6. Confirmar transação");
        println!("7. Reverter transação");
        println!("8. Sair");

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();

        match choice {
            "1" => {
                print!("Digite o nome do banco de dados: ");
                io::stdout().flush().unwrap();
                let mut db_name = String::new();
                io::stdin().read_line(&mut db_name).unwrap();
                let db_name = db_name.trim();

                print!("Digite a chave de criptografia: ");
                io::stdout().flush().unwrap();
                let mut encryption_key = String::new();
                io::stdin().read_line(&mut encryption_key).unwrap();
                let encryption_key = encryption_key.trim();

                let command = ClientCommand::Connect {
                    db_name: db_name.to_string(),
                    encryption_key: encryption_key.to_string(),
                };
                let response = client.send_command(command);
                println!("Resposta do servidor: {}", response);
            }
            "2" => {
                print!("Digite o comando SQL: ");
                io::stdout().flush().unwrap();
                let mut sql = String::new();
                io::stdin().read_line(&mut sql).unwrap();
                let sql = sql.trim();

                let command = ClientCommand::Execute {
                    command: sql.to_string(),
                };
                let response = client.send_command(command);
                println!("Resposta do servidor: {}", response);
            }
            "3" => {
                print!("Digite o nome do banco de dados para backup: ");
                io::stdout().flush().unwrap();
                let mut db_name = String::new();
                io::stdin().read_line(&mut db_name).unwrap();
                let db_name = db_name.trim();

                let command = ClientCommand::Backup {
                    db_name: db_name.to_string(),
                };
                let response = client.send_command(command);
                println!("Resposta do servidor: {}", response);
            }
            "4" => {
                print!("Digite o nome do banco de dados para restauração: ");
                io::stdout().flush().unwrap();
                let mut db_name = String::new();
                io::stdin().read_line(&mut db_name).unwrap();
                let db_name = db_name.trim();

                print!("Digite o caminho do arquivo de backup: ");
                io::stdout().flush().unwrap();
                let mut backup_file = String::new();
                io::stdin().read_line(&mut backup_file).unwrap();
                let backup_file = backup_file.trim();

                let command = ClientCommand::Restore {
                    db_name: db_name.to_string(),
                    backup_file: backup_file.to_string(),
                };
                let response = client.send_command(command);
                println!("Resposta do servidor: {}", response);
            }
            "5" => {
                let command = ClientCommand::BeginTransaction;
                let response = client.send_command(command);
                println!("Resposta do servidor: {}", response);
            }
            "6" => {
                let command = ClientCommand::CommitTransaction;
                let response = client.send_command(command);
                println!("Resposta do servidor: {}", response);
            }
            "7" => {
                let command = ClientCommand::RollbackTransaction;
                let response = client.send_command(command);
                println!("Resposta do servidor: {}", response);
            }
            "8" => {
                println!("Encerrando...");
                break;
            }
            _ => println!("Opção inválida."),
        }
    }
}
