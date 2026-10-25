-- Buchungen: erfasste Stunden auf ein Projekt (Projekt 1:N Buchung)
CREATE TABLE buchung (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    projekt_id  UUID NOT NULL REFERENCES projekt(id) ON DELETE RESTRICT,
    zeit        DOUBLE PRECISION NOT NULL CHECK (zeit > 0),
    erfasst_am  TIMESTAMP NOT NULL DEFAULT now()
);

CREATE INDEX buchung_projekt_id_idx ON buchung(projekt_id);
