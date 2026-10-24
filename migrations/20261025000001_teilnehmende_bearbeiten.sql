-- Aus „Teilnehmende hinzufügen“ wird „Teilnehmende bearbeiten“ (hinzufügen und entfernen).
UPDATE rolle SET rechte = array_replace(rechte, 'termine.teilnehmende_hinzufuegen', 'termine.teilnehmende_bearbeiten');
UPDATE rolle SET rechte = array_replace(rechte, 'anmeldung.teilnehmende_hinzufuegen', 'anmeldung.teilnehmende_bearbeiten');
