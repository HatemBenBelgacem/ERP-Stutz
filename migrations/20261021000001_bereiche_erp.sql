-- Die Übersicht gibt es nicht mehr; neue Bereiche Einkauf, Projekte und
-- Finanzen vergibt ein Admin bei Bedarf über die Rollen.
UPDATE rolle SET rechte = array_remove(rechte, 'uebersicht.sehen');
UPDATE rolle SET beschreibung = 'Termine, Kurse und Mitarbeiter (Personal) verwalten'
    WHERE name = 'Sachbearbeitung' AND beschreibung = 'Termine, Kurse und Mitarbeiter verwalten';
