-- Artikel: Bezeichnung, Grösse (S, M, L, XL) und optional ein Bild.
-- Das Bild liegt als Datei in bild_daten (BYTEA), bild enthält den Dateinamen.
-- Beide sind gemeinsam leer oder gemeinsam gefüllt.
CREATE TYPE artikel_groesse AS ENUM ('S', 'M', 'L', 'XL');

CREATE TABLE artikel (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bezeichnung  TEXT NOT NULL CHECK (btrim(bezeichnung) <> ''),
    bild         TEXT,
    bild_daten   BYTEA,
    groesse      artikel_groesse NOT NULL,
    erstellt_am  TIMESTAMP NOT NULL DEFAULT now(),
    geaendert_am TIMESTAMP NOT NULL DEFAULT now(),
    CHECK ((bild IS NULL) = (bild_daten IS NULL))
);
