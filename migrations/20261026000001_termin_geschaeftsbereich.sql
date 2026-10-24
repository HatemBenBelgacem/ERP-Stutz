-- Geschäftsbereiche pro Termin (M:N). Ein neuer Termin übernimmt die
-- Bereiche seines Kurses, danach können sie abweichen.
CREATE TABLE termin_geschaeftsbereich (
    termin_id            UUID NOT NULL REFERENCES termin(id) ON DELETE CASCADE,
    geschaeftsbereich_id UUID NOT NULL REFERENCES geschaeftsbereich(id) ON DELETE CASCADE,
    PRIMARY KEY (termin_id, geschaeftsbereich_id)
);

CREATE INDEX termin_geschaeftsbereich_geschaeftsbereich_id_idx ON termin_geschaeftsbereich(geschaeftsbereich_id);

-- Bestehende Termine bekommen die Bereiche ihres Kurses
INSERT INTO termin_geschaeftsbereich (termin_id, geschaeftsbereich_id)
SELECT t.id, kg.geschaeftsbereich_id
FROM termin t JOIN kurs_geschaeftsbereich kg ON kg.kurs_id = t.kurs_id;
