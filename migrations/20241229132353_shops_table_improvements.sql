-- Add migration script here

ALTER TABLE shop ADD COLUMN scrape_url TEXT NOT NULL;
ALTER TABLE shop ADD COLUMN scrape_impl TEXT NOT NULL;


INSERT INTO shop (name, scrape_url, scrape_impl) VALUES
  ("barbora", "https://barbora.ee/liha-kala-valmistoit/liha/veis-ja-muu-varske-liha", "barbora"),
  ("rimi", "https:/www.rimi.ee//epood/ee/tooted/liha--ja-kalatooted/veise--lamba--ja-ulukiliha/c/SH-8-21", "rimi"),
  ("selver", "https://www.selver.ee//liha-ja-kalatooted/veise-lamba-ja-ulukiliha?product_segment=4725", "selver");

