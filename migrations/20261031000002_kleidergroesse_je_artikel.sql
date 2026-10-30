-- Grössen gehören zum Artikel (1:N, F-015). Jede bisherige Zuordnung wird zu
-- einer eigenen Grösse des Artikels; danach entfallen die gemeinsamen Vorlagen
-- und die Zuordnung am Artikel.
ALTER TABLE kleidergroesse DROP CONSTRAINT kleidergroesse_bezeichnung_key;

ALTER TABLE kleidergroesse
    ADD COLUMN artikel_id UUID REFERENCES artikel(id) ON DELETE CASCADE;

INSERT INTO kleidergroesse (artikel_id, bezeichnung, sortierung)
SELECT a.id, k.bezeichnung, k.sortierung
FROM artikel a
JOIN kleidergroesse k ON k.id = a.kleidergroesse_id;

ALTER TABLE artikel
    DROP COLUMN kleidergroesse_id;

DELETE FROM kleidergroesse
WHERE artikel_id IS NULL;

ALTER TABLE kleidergroesse
    ALTER COLUMN artikel_id SET NOT NULL,
    ADD CONSTRAINT kleidergroesse_artikel_id_bezeichnung_key UNIQUE (artikel_id, bezeichnung);

CREATE INDEX kleidergroesse_artikel_id_idx ON kleidergroesse(artikel_id);
