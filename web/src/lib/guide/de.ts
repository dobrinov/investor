import { brokerExample, packExample } from './en'
import type { Section } from './types'

export const de: Section[] = [
  {
    id: 'start',
    title: 'Erste Schritte',
    blocks: [
      { p: 'Investor bewahrt Ihre Bücher, Pläne und Steuerzahlen in einer verschlüsselten Datei auf Ihrem Computer auf. Ein guter Einstieg:' },
      {
        steps: [
          'Exportieren Sie Ihre Auszüge bei jedem Broker (siehe die nächsten Abschnitte).',
          'Importieren Sie sie unter „Auszüge importieren“. Jeder Kauf, jede Dividende, Gebühr und Steuer wird zu einem ausgeglichenen Buchungssatz.',
          'Sehen Sie sich die „Übersicht“ an. Positionen ohne Kurs zählen zum Einstandspreis; einen Kurs ergänzen Sie mit „Kurs aktualisieren“.',
          'Füllen Sie Ihr Profil aus und legen Sie unter „Ziele“ Ihre Ziele an.',
          'Schauen Sie sich die „Prognosen“ an und vor der Steuererklärung „Steuern & Berichte“.',
          'Speichern Sie Ihre .invest-Datei und bewahren Sie eine Sicherungskopie auf.',
        ],
      },
    ],
  },
  {
    id: 'file',
    title: 'Ihre Datei, Ihr Passwort',
    blocks: [
      { p: 'Es gibt kein Konto und keinen Server. Alles steckt in Ihrer .invest-Datei, verschlüsselt mit einem Schlüssel aus Ihrem Passwort. Ohne das Passwort ist die Datei unlesbar – für jeden, auch für uns.' },
      {
        list: [
          'Bewahren Sie die Datei im Ordner „Dokumente“ auf und eine Kopie auf einem USB-Stick oder in einem Cloud-Ordner, den Sie ohnehin nutzen. Die Kopie bleibt verschlüsselt.',
          'Das Passwort lässt sich nicht zurücksetzen. Notieren Sie es an einem sicheren Ort oder nutzen Sie einen Passwort-Manager.',
          'Ungespeicherte Änderungen werden verschlüsselt in diesem Browser aufbewahrt, sodass ein Neuladen sie nicht verliert. Wenn Sie Ihre Browserdaten löschen, ist diese Kopie weg – speichern Sie die Datei, um Ihre Arbeit zu behalten.',
          'Eine Passwortänderung („Einstellungen“) gilt nur für danach gespeicherte Kopien. Ältere Kopien öffnen sich weiterhin mit dem alten Passwort, ersetzen Sie sie also.',
        ],
      },
    ],
  },
  {
    id: 'example',
    title: 'Zuerst das Beispielportfolio ansehen',
    blocks: [
      { p: 'Das Beispiel ist ein ausgedachter Anleger mit Indexfonds, Aktien, einem Rentenfonds, Gold an der Börse und als Münzen, Krypto, einem Sparkonto, einer Startup-Beteiligung und Zielen. Jede Ansicht funktioniert, sodass Sie sehen, was die App kann, bevor Sie etwas importieren.' },
      { list: ['Öffnen Sie es auf dem Startbildschirm mit „Beispielportfolio ansehen“ oder über die Adresse mit der Endung #example.', 'Ändern Sie, was Sie möchten: Nichts wird gespeichert, und die Kopie Ihrer eigenen Datei in diesem Browser bleibt unberührt.', '„Beispiel schließen“ bringt Sie zurück zum Startbildschirm.'] },
    ],
  },
  {
    id: 'bank',
    title: 'Bankkonten',
    blocks: [
      { p: 'Für Spar- und Girokonten gibt es keinen Auszug zum Importieren, daher tragen Sie den Kontostand ab und zu selbst ein. Das Geld zählt als Guthaben: Es füllt Ihren Notgroschen und erscheint in Nettovermögen und Prognosen.' },
      { steps: ['Wählen Sie in der „Übersicht“ „Bankkonto hinzufügen“ und geben Sie einen Namen ein, etwa „UBB Sparkonto“, dazu die Währung und den heutigen Kontostand.', 'Ändert sich der Kontostand, wählen Sie „Stand aktualisieren“ und geben den vollständigen neuen Stand ein. Die Bücher erfassen nur die Differenz, als Einzahlung oder Entnahme.', 'Bei einem Konto in einer anderen Währung geben Sie den Wechselkurs des Tages an.'] },
    ],
  },
  {
    id: 'metals',
    title: 'Physisches Gold und Silber',
    blocks: [
      { p: 'Münzen und Barren, die Sie selbst aufbewahren, werden in Gramm Feinmetall gezählt, sodass Einstand, Wert und Gewinne wie bei Aktien funktionieren. Ein an der Börse gehandeltes Goldprodukt wie Xetra-Gold ist dagegen eine normale Position: Kaufen Sie es unter „Order planen“.' },
      { steps: ['Wählen Sie in der „Übersicht“ „Münzen oder Barren hinzufügen“. Geben Sie das Feingewicht in Feinunzen oder Gramm ein (eine Unzen-Münze enthält eine Unze, auch wenn sie mehr wiegt) und alles, was Sie bezahlt haben, einschließlich des Händleraufschlags.', 'Aktualisieren Sie den Preis ab und zu, pro Unze oder pro Gramm. Bis dahin wird der bezahlte Preis verwendet.', 'Wenn Sie verkaufen, erfassen Sie, was Sie nach der Händlermarge erhalten haben. Der Einstand wird zuerst von Ihren ältesten Käufen abgezogen.'] },
      { p: 'In der Steueransicht zeigt jeder Verkauf, wann das Metall gekauft wurde, denn mehrere Länder besteuern Gold nach der Haltedauer. In Bulgarien behandeln es die Entwurfsregeln als steuerfrei; klären Sie das mit einem Steuerberater.' },
    ],
  },
  {
    id: 'private',
    title: 'Private Beteiligungen und ihre Auszahlung',
    blocks: [
      { p: 'Geld in etwas ohne Marktpreis, etwa eine Mitarbeiterbeteiligung oder ein Anteil an einer privaten Firma. Es zählt zum Nettovermögen mit dem bezahlten Betrag oder einem Wert, den Sie festlegen, und bleibt außerhalb der Prognosen, weil Sie es nicht jederzeit verkaufen können.' },
      { steps: ['Wählen Sie in der „Übersicht“ „Private Beteiligung hinzufügen“: was Sie bezahlt haben, wann, in welcher Währung und zu welchem Wechselkurs und wie das zurückfließende Geld besteuert wird.', 'Legen Sie einen Wert fest, sobald Sie erfahren, was die Beteiligung wert ist.', 'Wenn das Geld zurückkommt, wählen Sie „Auszahlung erfassen“: den Betrag vor Steuern, die von Ihrem Arbeitgeber einbehaltene Steuer und ob es als Arbeitslohn oder als Veräußerungsgewinn besteuert wurde.'] },
      { p: 'Die Position wird dann geschlossen. Ihre Einzahlung fließt an Sie zurück, und die Differenz wird als Einkommen oder Gewinn gebucht. Arbeitslohn erhält in der Steueransicht einen eigenen Abschnitt; ein Veräußerungsgewinn erscheint bei Ihren übrigen Verkäufen. Ging das Geld auf ein Bankkonto, das Sie hier führen, aktualisieren Sie auch dessen Kontostand. „Auszahlung zurücknehmen“ macht sie wieder rückgängig.' },
    ],
  },
  {
    id: 'claude',
    title: 'Einen KI-Assistenten fragen',
    blocks: [
      { p: 'Ein KI-Assistent kann Ihr Portfolio lesen, Fragen mit den Zahlen der App beantworten, Ihnen beim Planen helfen und Ihre Entscheidungen erfassen, sobald Sie zustimmen. Er ist optional: Alles in Investor funktioniert auch ohne ihn.' },
      { p: 'Drücken Sie in der Desktop-App die Schaltfläche ✦ unten rechts und wählen Sie ein Modell: eines auf Ihrem Computer über Ollama oder LM Studio (nichts verlässt Ihren Computer) oder Anthropic bzw. OpenAI mit Ihrem eigenen API-Schlüssel. Jede Änderung, die er vornehmen möchte, erscheint als Karte zur Freigabe und wird zu einer ungespeicherten Änderung, die Sie wie gewohnt speichern. Auf der Website erklärt die Schaltfläche, warum die Desktop-App nötig ist: Ihr Portfolio sollte auf dem Weg zu einem Modell nicht über eine Webseite laufen.' },
      { p: 'Sie können auch Ihr eigenes KI-Tool nutzen, etwa Claude Code, Cursor oder VS Code. Öffnen Sie in der Desktop-App den Assistenten, drücken Sie oben die Stecker-Schaltfläche und wählen Sie „Einschalten“. Kopieren Sie den angezeigten Befehl in Claude Code oder die Adresse in die MCP-Einstellungen eines anderen Tools. Das ist alles: keine Installation, kein Passwort zu speichern.' },
      { list: ['Es funktioniert, solange die App offen und Ihre Datei entsperrt ist. Sperren Sie die Datei oder schließen Sie die App, kann Ihr Tool sie nicht mehr lesen.', 'Jede Änderung, die Ihr Tool vornehmen möchte, erscheint hier als Karte zur Freigabe und wird zu einer ungespeicherten Änderung, die Sie wie gewohnt speichern.', 'Die Adresse enthält ein Geheimnis. Erstellen Sie eine neue Adresse, falls Sie sie versehentlich weitergegeben haben. Bei einem Cloud-Modell wird, was Ihr Tool liest, an diesen Anbieter gesendet.', 'In Claude Code fügt /plugin install investor --marketplace dobrinov/investor fertige Skills hinzu: Monats-Check, Order durchspielen, Steuersaison und Ruhestand.'] },
      { p: 'Mögliche Fragen:' },
      { list: ['„Was besitze ich, und wie viel davon ist Guthaben?“', '„Vergleiche, 10.000 € auf einmal anzulegen, mit einer Verteilung über 12 Monate, über 20 Jahre.“', '„Ich habe heute 2 VUAA auf Xetra zu 131,20 verkauft. Erfasse das und nenne mir die Steuer.“', '„Bin ich bei meinen Zielen im Plan? Was bräuchte es, um mit 60 in den Ruhestand zu gehen?“'] },
    ],
  },
  {
    id: 'export-ibkr',
    title: 'Export aus Interactive Brokers',
    blocks: [
      { p: 'Investor liest eine Activity Flex Query im CSV-Format. Sie richten sie einmal ein und führen sie aus, wann immer Sie aktuelle Daten möchten.' },
      {
        steps: [
          'Öffnen Sie im IBKR Portal Performance & Reports → Flex Queries.',
          'Drücken Sie neben Activity Flex Query auf + und nennen Sie sie „investor-export“.',
          'Haken Sie diese Abschnitte an und drücken Sie in jedem Select All: Account Information, Trades (Executions), Cash Transactions, Transaction Fees, Open Positions (Summary), Corporate Actions, Transfers, Financial Instrument Information, Conversion Rates.',
          'Delivery configuration: Format CSV; Include header and trailer records – No; Include column headers – Yes; Include section code and line descriptor – Yes; Date format yyyy-MM-dd; Time format HH:mm:ss; Date/time separator „;“.',
          'Speichern Sie und drücken Sie Run. Wählen Sie Custom Date Range von Ihrer ersten Einzahlung bis heute. Begrenzt IBKR den Zeitraum auf 365 Tage, führen Sie sie einmal pro Jahr aus.',
          'Importieren Sie die heruntergeladene Datei hier. Ein späteres erneutes Ausführen ist unbedenklich: Bereits importierte Zeilen werden übersprungen.',
        ],
      },
      { p: 'Wenn Sie nur die offenen Positionen exportieren, kommen die Bestände als Anfangsbestände ohne Kaufdatum und Gebühren an. Importieren Sie später die vollständige Historie, werden sie automatisch ersetzt.' },
    ],
  },
  {
    id: 'export-revolut',
    title: 'Export aus Revolut',
    blocks: [
      {
        steps: [
          'Öffnen Sie in der Revolut-App Invest (oder Stocks).',
          'Öffnen Sie ⋯ oder Documents → Brokerage account → Account statement.',
          'Wählen Sie Excel/CSV und den gesamten Zeitraum und importieren Sie die Datei dann hier.',
        ],
      },
      { p: 'Revolut-Auszüge geben nicht an, an welcher Börse eine Order ausgeführt wurde. In Bulgarien entscheidet das darüber, ob ein Gewinn steuerfrei ist, daher werden solche Verkäufe in der Steueransicht mit „Platz prüfen“ markiert.' },
    ],
  },
  {
    id: 'add-broker',
    title: 'Ihr Broker oder Ihre Börse fehlt?',
    blocks: [
      { p: 'Jeder Broker wird durch eine kleine Konfigurationsdatei beschrieben, zum Hinzufügen braucht es also keine Programmierung. Es gibt zwei Wege.' },
      { p: 'Fragen Sie danach. Eröffnen Sie ein Issue auf GitHub mit dem Namen des Brokers und einem anonymisierten Export:' },
      {
        steps: [
          'Exportieren Sie einen Auszug, der möglichst mindestens eine Einzahlung, einen Kauf, einen Verkauf, eine Dividende und eine Gebühr enthält.',
          'Ersetzen Sie Kontonummer, Namen und Beträge durch ausgedachte Werte. Lassen Sie Spaltennamen, Datumsformat und Typbezeichnungen genau wie exportiert – genau die liest die Konfiguration.',
          'Eröffnen Sie ein neues Issue auf GitHub und hängen Sie die Datei an.',
        ],
      },
      { link: 'Issue auf GitHub eröffnen', href: 'https://github.com/dobrinov/investor/issues/new' },
      { p: 'Fügen Sie ihn selbst hinzu. Schreiben Sie eine Konfigurationsdatei wie diese, legen Sie sie in den Ordner brokers/ des Projekts und eröffnen Sie einen Pull Request. Zeilen, die Ihre Konfiguration nicht erkennt, werden beim Import mit Zeilennummer aufgelistet, sodass Sie sehen, was noch fehlt.' },
      { code: brokerExample },
      { p: 'Zeilenarten: buy, sell, deposit, withdrawal, cash_transfer, dividend, withholding_tax, fee, interest, fx_pair_trade, position_snapshot, ignore. Gebührenarten: commission, fx_conversion, exchange, transaction_tax, custody, interest, other.' },
    ],
  },
  {
    id: 'add-tax',
    title: 'Ihre Steuerbehörde fehlt?',
    blocks: [
      { p: 'Steuerregeln kommen in Paketen: eine Datei pro Land und Steuerjahr im Ordner jurisdictions/ des Projekts (zum Beispiel jurisdictions/bg/2026.toml). Ein Paket legt Steuersätze, Befreiungen, Formulare und Fristen fest.' },
      {
        steps: [
          'Eröffnen Sie ein Issue auf GitHub mit Land und Steuerjahr und verlinken Sie die offiziellen Hinweise der Steuerbehörde.',
          'Oder kopieren Sie das bulgarische Paket, ändern Sie die Werte und eröffnen Sie einen Pull Request.',
          'Lassen Sie verified = false stehen, bis ein Steuerberater das Paket geprüft hat. Die App zeigt bei ungeprüften Paketen eine Warnung.',
        ],
      },
      { code: packExample },
      { p: 'Die Werte oben sind Platzhalter. Jeder Steuersatz, jede Befreiung und jeder Formularname muss aus den offiziellen Quellen stammen.' },
      { link: 'Issue auf GitHub eröffnen', href: 'https://github.com/dobrinov/investor/issues/new' },
    ],
  },
  {
    id: 'glossary',
    title: 'Begriffe in der App',
    blocks: [
      {
        terms: [
          ['Doppelte Buchführung', 'Jede Geldbewegung wird zweimal erfasst – woher sie kam und wohin sie ging –, sodass die Bücher immer ausgeglichen sind.'],
          ['Buchungssatz', 'Ein Ereignis, etwa ein Kauf oder eine Dividende, erfasst als Buchungen, die zusammen null ergeben.'],
          ['Saldenliste', 'Der Saldo jedes Kontos. Stimmen die Bücher, ist Soll gleich Haben.'],
          ['Anschaffungskosten', 'Was Sie für eine Anlage bezahlt haben, einschließlich Gebühren, soweit die Regeln das erlauben.'],
          ['FIFO', 'First in, first out: Beim Verkauf gelten die zuerst gekauften Stücke als zuerst verkauft.'],
          ['Realisierter Gewinn', 'Gewinn oder Verlust, der durch einen Verkauf feststeht. Er wird besteuert.'],
          ['Nicht realisierter Gewinn', 'Gewinn oder Verlust auf Anlagen, die Sie noch halten, zum letzten Kurs.'],
          ['TER', 'Gesamtkostenquote: die laufenden jährlichen Kosten eines Fonds, die aus seinem Kurs entnommen werden.'],
          ['Quellensteuer', 'Steuer, die ein anderes Land einbehält, bevor eine Dividende bei Ihnen ankommt. Wird im Inland oft teilweise angerechnet.'],
          ['Handelsplatz', 'Die Börse, an der eine Order ausgeführt wurde, etwa Xetra oder NASDAQ.'],
          ['ETF', 'Ein börsengehandelter Fonds, der wie eine Aktie gehandelt wird und meist viele Unternehmen auf einmal hält.'],
          ['Notgroschen', 'Geld, das als Guthaben für Unvorhergesehenes bereitliegt, damit Sie nie zu einem ungünstigen Zeitpunkt Anlagen verkaufen müssen.'],
        ],
      },
    ],
  },
]
