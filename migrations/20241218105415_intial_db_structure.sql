CREATE TABLE shop (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  name TEXT UNIQUE
);

CREATE TABLE product (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  name TEXT,
  embedding FLOAT[1024],
  embedding_model TEXT,
  beef_cut_id INTEGER,
  beef_cut_guess_confidence INTEGER,
  inserted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
  FOREIGN KEY(beef_cut_id) REFERENCES beef_cut(id)
);

CREATE TABLE beef_cut (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  name TEXT
);

CREATE TABLE product_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  shop_id INTEGER NOT NULL,
  product_id INTEGER NOT NULL,
  price INTEGER NOT NULL,
  inserted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
  FOREIGN KEY(shop_id) REFERENCES shop(id),
  FOREIGN KEY(product_id) REFERENCES product(id)
);
