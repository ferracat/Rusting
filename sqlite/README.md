

```bash
sudo apt install libsqlite3-dev

sqlite3 my_database.db

sqlite> .tables
person
sqlite> SELECT name FROM sqlite_master WHERE type='table';
person
sqlite> .mode column
sqlite> .headers on
sqlite> SELECT * FROM person;
id  name   age
--  -----  ---
1   Alice  30
2   Bob    25
sqlite>
```

