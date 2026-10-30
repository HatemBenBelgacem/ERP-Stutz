-- Kleidergrösse als eigene Tabelle statt Enum (F-015). Ein Artikel gehört zu
-- genau einer Kleidergrösse (N:1). Die bisherigen Werte S, M, L, XL werden
-- als Kleidergrössen übernommen und den Artikeln zugeordnet.
CREATE TABLE kleidergroesse (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bezeichnung TEXT NOT NULL UNIQUE CHECK (btrim(bezeichnung) <> ''),
    sortierung  INTEGER NOT NULL
);

INSERT INTO kleidergroesse (bezeichnung, sortierung)
VALUES ('S', 1), ('M', 2), ('L', 3), ('XL', 4);

ALTER TABLE artikel
    ADD COLUMN kleidergroesse_id UUID REFERENCES kleidergroesse(id) ON DELETE RESTRICT;

UPDATE artikel
SET kleidergroesse_id = k.id
FROM kleidergroesse k
WHERE k.bezeichnung = artikel.groesse::text;

ALTER TABLE artikel
    ALTER COLUMN kleidergroesse_id SET NOT NULL,
    DROP COLUMN groesse;

DROP TYPE artikel_groesse;

CREATE INDEX artikel_kleidergroesse_id_idx ON artikel(kleidergroesse_id);
