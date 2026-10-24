-- Bauobjekte: Bauherr und Architekt sind Adressen (je N:1).
CREATE TABLE bauobjekt (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bezeichnung  TEXT NOT NULL,
    bauherr_id   UUID REFERENCES adresse(id) ON DELETE SET NULL,
    architekt_id UUID REFERENCES adresse(id) ON DELETE SET NULL
);

CREATE INDEX bauobjekt_bauherr_id_idx ON bauobjekt(bauherr_id);
CREATE INDEX bauobjekt_architekt_id_idx ON bauobjekt(architekt_id);

-- Projekte gehören zu einem Bauobjekt (Bauobjekt 1:N Projekt). Bauherr und
-- Architekt werden beim Anlegen vom Bauobjekt übernommen und sind danach
-- pro Projekt anpassbar.
CREATE TABLE projekt (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    projektnummer TEXT NOT NULL UNIQUE,
    bezeichnung   TEXT NOT NULL,
    bauobjekt_id  UUID NOT NULL REFERENCES bauobjekt(id) ON DELETE RESTRICT,
    bauherr_id    UUID REFERENCES adresse(id) ON DELETE SET NULL,
    architekt_id  UUID REFERENCES adresse(id) ON DELETE SET NULL
);

CREATE INDEX projekt_bauobjekt_id_idx ON projekt(bauobjekt_id);
CREATE INDEX projekt_bauherr_id_idx ON projekt(bauherr_id);
CREATE INDEX projekt_architekt_id_idx ON projekt(architekt_id);
