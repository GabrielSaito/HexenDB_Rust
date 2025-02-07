![HexenDB Logo](https://github.com/GabrielSaito/HexenDB_Rust/blob/master/HexenDbLogo.png?raw=true)

**HexenDB** é um banco de dados simples desenvolvido em Rust com propósitos de aprendizagem. Ele explora conceitos fundamentais de gerenciamento de dados, como criação de tabelas, inserção de registros, transações, backups criptografados e restauração. Este projeto não é funcional para uso em produção e foi criado exclusivamente para fins de estudo.

---

## **Índice**

- [Uso](#uso)
  - [Comandos SQL](#comandos-sql)
  - [Backup e Restauração](#backup-e-restauração)
  - [Transações](#transações)
- [Exemplos](#exemplos)
- [Limitações](#limitações)

---

## **Uso** <a id="uso"></a>

O **HexenDB** suporta uma variedade de operações básicas para manipulação de dados. Abaixo estão as principais funcionalidades.

### **Comandos SQL** <a id="comandos-sql"></a>

O **HexenDB** suporta os seguintes comandos SQL:

#### Conectar ao Banco de Dados
```sql
CONNECT <db_name> <encryption_key>
```
Exemplo:
```sql
CONNECT meu_banco minha_chave_secreta
```

#### Criar uma Tabela
```sql
CREATE TABLE <table_name> (<column_definitions>)
```
Exemplo:
```sql
CREATE TABLE usuarios (
    id PRIMARY KEY,
    nome,
    email FOREIGN KEY REFERENCES contatos(email)
)
```

#### Inserir Dados
```sql
INSERT INTO <table_name> VALUES (<values>)
```
Exemplo:
```sql
INSERT INTO usuarios VALUES (1, 'exemplo', 'exemplo@email.com')
```

#### Alterar uma Tabela
- Adicionar Coluna:
  ```sql
  ALTER TABLE <table_name> ADD COLUMN <column_name>
  ```
- Remover Coluna:
  ```sql
  ALTER TABLE <table_name> DROP COLUMN <column_name>
  ```

#### Excluir uma Tabela
```sql
DROP TABLE <table_name>
```

#### Consultar Dados
```sql
SELECT * FROM <table_name> [WHERE <condition>]
```
Exemplo:
```sql
SELECT * FROM usuarios WHERE nome = 'LittleHair'
```

---

### **Backup e Restauração** <a id="backup-e-restauração"></a>

#### Criar Backup
```sql
BACKUP <db_name>
```

#### Restaurar Backup
```sql
RESTORE <db_name> <backup_file>
```

---

### **Transações** <a id="transações"></a>

- Iniciar Transação:
  ```sql
  BEGIN TRANSACTION
  ```

- Confirmar Transação:
  ```sql
  COMMIT TRANSACTION
  ```

- Reverter Transação:
  ```sql
  ROLLBACK TRANSACTION
  ```

---

## **Exemplos** <a id="exemplos"></a>

### Fluxo Completo

1. Conecte ao banco de dados:
   ```sql
   CONNECT meu_banco minha_chave_secreta
   ```

2. Crie uma tabela:
   ```sql
   CREATE TABLE usuarios (
       id PRIMARY KEY,
       nome,
       email
   )
   ```

3. Insira dados:
   ```sql
   INSERT INTO usuarios VALUES (1, 'LittleHair', 'littleHair@email.com')
   ```

4. Consulte os dados:
   ```sql
   SELECT * FROM usuarios
   ```

5. Faça backup:
   ```sql
   BACKUP meu_banco
   ```

6. Restaure o backup:
   ```sql
   RESTORE meu_banco backups/bd_20231001120000.hxn.gz
   ```

---

## **Limitações** <a id="limitações"></a>

- Este projeto foi criado exclusivamente para fins educacionais e ainda está em desenvolvimento.
- Não é útil para uso em ambientes de produção.
- Ainda há funcionalidades limitadas, como suporte a consultas complexas (`JOIN`, `ORDER BY`, etc.).
