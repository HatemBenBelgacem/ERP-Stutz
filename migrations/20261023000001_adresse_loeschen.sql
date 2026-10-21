-- Adressen lassen sich auch löschen, wenn Termine sie nutzen;
-- diese Termine haben danach keine Adresse mehr.
ALTER TABLE termin
    DROP CONSTRAINT termin_adresse_id_fkey,
    ADD CONSTRAINT termin_adresse_id_fkey
        FOREIGN KEY (adresse_id) REFERENCES adresse(id) ON DELETE SET NULL;
