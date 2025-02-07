use crate::table::{Column, Table};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{decode, encode};
use flate2::write::GzEncoder;
use flate2::Compression;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

#[derive(Serialize, Deserialize)]
struct TransactionState {
    tables: HashMap<String, Table>,
}

pub struct Database {
    file_path: String,
    tables: HashMap<String, Table>,
    encryption_key: [u8; 32],
    charset: String,
    connected: bool,
    transaction_active: bool,
    transaction_state: Option<TransactionState>,
}

impl Database {
    pub fn new(file_path: String, encryption_key: &str, charset: &str) -> Self {
        let mut key = [0u8; 32];
        let encryption_key_bytes = encryption_key.as_bytes();
        key[..encryption_key_bytes.len().min(32)]
            .copy_from_slice(&encryption_key_bytes[..encryption_key_bytes.len().min(32)]);
        Database {
            file_path,
            tables: HashMap::new(),
            encryption_key: key,
            charset: charset.to_string(),
            connected: true,
            transaction_active: false,
            transaction_state: None,
        }
    }

    pub fn begin_transaction(&mut self) -> String {
        if self.transaction_active {
            return "Erro: Já existe uma transação ativa.".to_string();
        }

        self.transaction_state = Some(TransactionState {
            tables: self.tables.clone(),
        });
        self.transaction_active = true;
        "Transação iniciada com sucesso.".to_string()
    }

    pub fn commit_transaction(&mut self) -> String {
        if !self.transaction_active {
            return "Erro: Nenhuma transação ativa para confirmar.".to_string();
        }

        self.transaction_active = false;
        self.transaction_state = None;
        self.save_database();
        "Transação confirmada com sucesso.".to_string()
    }

    pub fn rollback_transaction(&mut self) -> String {
        if !self.transaction_active {
            return "Erro: Nenhuma transação ativa para reverter.".to_string();
        }

        if let Some(state) = self.transaction_state.take() {
            self.tables = state.tables;
        }
        self.transaction_active = false;
        "Transação revertida com sucesso.".to_string()
    }

    pub fn execute_command(&mut self, command: &str) -> String {
        if !self.connected {
            return "Erro: Você não está conectado ao banco de dados.".to_string();
        }

        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return "Comando inválido.".to_string();
        }

        match parts[0].to_uppercase().as_str() {
            "CREATE" if parts[1].to_uppercase() == "TABLE" => {
                if self.transaction_active {
                    self.execute_create_table(parts)
                } else {
                    "Erro: Operações devem ser executadas dentro de uma transação.".to_string()
                }
            }
            "INSERT" if parts[1].to_uppercase() == "INTO" => {
                if self.transaction_active {
                    self.execute_insert(parts)
                } else {
                    "Erro: Operações devem ser executadas dentro de uma transação.".to_string()
                }
            }
            "ALTER" if parts[1].to_uppercase() == "TABLE" => {
                if self.transaction_active {
                    self.execute_alter_table(parts)
                } else {
                    "Erro: Operações devem ser executadas dentro de uma transação.".to_string()
                }
            }
            "DROP" if parts[1].to_uppercase() == "TABLE" => {
                if self.transaction_active {
                    self.execute_drop_table(parts)
                } else {
                    "Erro: Operações devem ser executadas dentro de uma transação.".to_string()
                }
            }
            "SELECT" if parts[1].to_uppercase() == "*" && parts[2].to_uppercase() == "FROM" => {
                self.execute_select(parts)
            }
            _ => "Comando SQL não suportado.".to_string(),
        }
    }

    fn execute_create_table(&mut self, parts: Vec<&str>) -> String {
        let table_name = parts[2];
        let joined = parts[3..].join(" ");
        let column_definitions: Vec<&str> = joined
            .trim_matches(|c| c == '(' || c == ')' || c == ';')
            .split(',')
            .map(|s| s.trim())
            .collect();

        let mut columns = Vec::new();
        for def in column_definitions {
            let mut parts = def.split_whitespace();
            let column_name = parts.next().unwrap_or("").to_string();
            let mut is_primary_key = false;
            let mut foreign_key = None;

            for part in parts.clone() {
                match part.to_uppercase().as_str() {
                    "PRIMARY" if parts.clone().nth(1).unwrap_or("") == "KEY" => {
                        is_primary_key = true;
                    }
                    "FOREIGN" if parts.clone().nth(1).unwrap_or("") == "KEY" => {
                        let referenced_table = parts.clone().nth(3).unwrap_or("");
                        let referenced_column = parts.clone().nth(5).unwrap_or("");
                        foreign_key =
                            Some((referenced_table.to_string(), referenced_column.to_string()));
                    }
                    _ => {}
                }
            }

            columns.push(Column {
                name: column_name,
                is_primary_key,
                foreign_key,
            });
        }

        self.tables.insert(
            table_name.to_string(),
            Table {
                columns,
                data: Vec::new(),
            },
        );

        format!("Tabela '{}' criada com sucesso!", table_name)
    }

    fn execute_insert(&mut self, parts: Vec<&str>) -> String {
        let table_name = parts[2];
        let values: Vec<String> = parts[4..]
            .join(" ")
            .trim_matches(|c| c == '(' || c == ')' || c == ';')
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        if let Some(table) = self.tables.get(table_name) {
            for (i, column) in table.columns.iter().enumerate() {
                if let Some((referenced_table, referenced_column)) = &column.foreign_key {
                    if !self.is_valid_foreign_key(referenced_table, referenced_column, &values[i]) {
                        return format!(
                            "Erro: Valor '{}' inválido para a coluna '{}' na tabela '{}'.",
                            values[i], column.name, table_name
                        );
                    }
                }
            }
        }

        if let Some(table) = self.tables.get_mut(table_name) {
            table.data.push(values);
            return format!("Dados inseridos na tabela '{}'.", table_name);
        }

        format!("Erro: Tabela '{}' não existe.", table_name)
    }

    fn is_valid_foreign_key(
        &self,
        referenced_table: &str,
        referenced_column: &str,
        value: &str,
    ) -> bool {
        if let Some(table) = self.tables.get(referenced_table) {
            if let Some(column_index) = table
                .columns
                .iter()
                .position(|c| c.name == referenced_column)
            {
                return table.data.iter().any(|row| row[column_index] == value);
            }
        }
        false
    }

    fn execute_select(&self, parts: Vec<&str>) -> String {
        let table_name = parts[3];

        if let Some(table) = self.tables.get(table_name) {
            let mut condition_column = None;
            let mut condition_value = None;

            if parts.len() > 4 && parts[4].to_uppercase() == "WHERE" {
                if parts.len() < 6 {
                    return "Sintaxe inválida. Use: SELECT * FROM <table_name> WHERE <column> = <value>".to_string();
                }
                condition_column = Some(parts[5]);
                condition_value = Some(parts[7].trim_matches(|c| c == '\'' || c == '"'));
            }

            let header: Vec<String> = table.columns.iter().map(|c| c.name.clone()).collect();
            let mut result = format!(
                "+{}\n",
                "-".repeat(header.iter().map(|h| h.len() + 2).sum::<usize>() + header.len() - 1)
            );
            result += &format!("| {}\n", header.join(" | "));
            result += &format!(
                "+{}\n",
                "-".repeat(header.iter().map(|h| h.len() + 2).sum::<usize>() + header.len() - 1)
            );

            for row in &table.data {
                if let Some(column) = condition_column {
                    if let Some(value) = condition_value {
                        if let Some(index) = table.columns.iter().position(|c| c.name == column) {
                            if row[index] != value {
                                continue;
                            }
                        } else {
                            return format!("Erro: Coluna '{}' não encontrada.", column);
                        }
                    }
                }

                result += &format!("| {}\n", row.join(" | "));
            }

            result += &format!(
                "+{}\n",
                "-".repeat(header.iter().map(|h| h.len() + 2).sum::<usize>() + header.len() - 1)
            );
            return result;
        }

        format!("Erro: Tabela '{}' não existe.", table_name)
    }

    fn execute_alter_table(&mut self, parts: Vec<&str>) -> String {
        let table_name = parts[2];
        let action = parts[3].to_uppercase();

        if let Some(table) = self.tables.get_mut(table_name) {
            match action.as_str() {
                "ADD" => {
                    if parts.len() < 5 || parts[4].to_uppercase() != "COLUMN" {
                        return "Sintaxe inválida. Use: ALTER TABLE <nome> ADD COLUMN <coluna>"
                            .to_string();
                    }
                    let column_name = parts[5];
                    table.columns.push(Column {
                        name: column_name.to_string(),
                        is_primary_key: false,
                        foreign_key: None,
                    });
                    format!(
                        "Coluna '{}' adicionada à tabela '{}'.",
                        column_name, table_name
                    )
                }
                "DROP" => {
                    if parts.len() < 5 || parts[4].to_uppercase() != "COLUMN" {
                        return "Sintaxe inválida. Use: ALTER TABLE <nome> DROP COLUMN <coluna>"
                            .to_string();
                    }
                    let column_name = parts[5];
                    if let Some(index) = table.columns.iter().position(|c| c.name == column_name) {
                        table.columns.remove(index);
                        for row in &mut table.data {
                            row.remove(index);
                        }
                        return format!(
                            "Coluna '{}' removida da tabela '{}'.",
                            column_name, table_name
                        );
                    }
                    format!(
                        "Erro: Coluna '{}' não encontrada na tabela '{}'.",
                        column_name, table_name
                    )
                }
                _ => "Ação inválida. Use ADD COLUMN ou DROP COLUMN.".to_string(),
            }
        } else {
            format!("Erro: Tabela '{}' não existe.", table_name)
        }
    }

    fn execute_drop_table(&mut self, parts: Vec<&str>) -> String {
        let table_name = parts[2];
        if self.tables.remove(table_name).is_some() {
            format!("Tabela '{}' excluída com sucesso.", table_name)
        } else {
            format!("Erro: Tabela '{}' não existe.", table_name)
        }
    }

    pub fn create_backup(
        db_name: &str,
        databases: &Arc<Mutex<HashMap<String, Database>>>,
    ) -> Result<String, String> {
        let databases = databases.lock().unwrap();
        let db = databases
            .get(db_name)
            .ok_or("Banco de dados não encontrado.".to_string())?;

        let backup_dir = "backups";
        if !Path::new(backup_dir).exists() {
            fs::create_dir(backup_dir)
                .map_err(|e| format!("Erro ao criar diretório de backups: {}", e))?;
        }

        let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
        let backup_file = format!("{}/{}_{}.hxn.gz", backup_dir, db_name, timestamp);

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        let serialized_data = serde_json::to_string(&db.tables)
            .map_err(|e| format!("Erro ao serializar dados: {}", e))?;
        encoder
            .write_all(serialized_data.as_bytes())
            .map_err(|e| format!("Erro ao escrever backup: {}", e))?;
        let compressed_data = encoder
            .finish()
            .map_err(|e| format!("Erro ao compactar backup: {}", e))?;

        let mut file = File::create(&backup_file)
            .map_err(|e| format!("Erro ao criar arquivo de backup: {}", e))?;
        file.write_all(&compressed_data)
            .map_err(|e| format!("Erro ao salvar backup: {}", e))?;

        Ok(format!("Backup criado com sucesso: {}", backup_file))
    }

    pub fn restore_backup(
        db_name: &str,
        backup_file: &str,
        databases: &Arc<Mutex<HashMap<String, Database>>>,
    ) -> Result<String, String> {
        let mut databases = databases.lock().unwrap();
        if !Path::new(backup_file).exists() {
            return Err("Arquivo de backup não encontrado.".to_string());
        }

        let mut file = File::open(backup_file)
            .map_err(|e| format!("Erro ao abrir arquivo de backup: {}", e))?;
        let mut compressed_data = Vec::new();
        file.read_to_end(&mut compressed_data)
            .map_err(|e| format!("Erro ao ler arquivo de backup: {}", e))?;

        let mut decoder = flate2::read::GzDecoder::new(&compressed_data[..]);
        let mut decompressed_data = String::new();
        decoder
            .read_to_string(&mut decompressed_data)
            .map_err(|e| format!("Erro ao descompactar backup: {}", e))?;

        let tables: HashMap<String, Table> = serde_json::from_str(&decompressed_data)
            .map_err(|e| format!("Erro ao desserializar dados: {}", e))?;
        databases.insert(
            db_name.to_string(),
            Database {
                file_path: format!("{}.hxn", db_name),
                tables,
                encryption_key: [0u8; 32],
                charset: "UTF-8".to_string(),
                connected: true,
                transaction_active: false,
                transaction_state: None,
            },
        );

        Ok(format!("Backup restaurado com sucesso para '{}'.", db_name))
    }

    fn save_database(&self) {
        let serialized_data = serde_json::to_string(&self.tables).unwrap();
        let encrypted_data = self.encrypt_data(serialized_data.as_bytes());
        fs::write(&self.file_path, encrypted_data).unwrap();
    }

    fn encrypt_data(&self, data: &[u8]) -> Vec<u8> {
        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key).unwrap();
        let nonce = Nonce::from_slice(b"unique_nonce_123");
        cipher.encrypt(nonce, data).unwrap()
    }
}
