import { brokerExample, packExample } from './en'
import type { Section } from './types'

export const nl: Section[] = [
  {
    id: 'start',
    title: 'Aan de slag',
    blocks: [
      { p: 'Investor bewaart je boekhouding, plannen en belastingcijfers in één versleuteld bestand op je computer. Een goede eerste keer:' },
      {
        steps: [
          'Exporteer je afschriften bij elke broker (zie de volgende onderdelen).',
          'Importeer ze op het scherm “Afschriften importeren”. Elke transactie, elk dividend, elke kostenpost en belasting wordt een sluitende journaalpost.',
          'Bekijk het Overzicht. Posities zonder koers tellen tegen wat je betaalde; voeg een koers toe met “Koers bijwerken”.',
          'Vul je profiel in en voeg doelen toe op het scherm Doelen.',
          'Verken Prognoses, en Belasting & aangifte voordat je aangifte doet.',
          'Sla je .invest-bestand op en bewaar een back-up.',
        ],
      },
    ],
  },
  {
    id: 'file',
    title: 'Je bestand, je wachtwoord',
    blocks: [
      { p: 'Er is geen account en geen server. Alles staat in je .invest-bestand, versleuteld met een sleutel die uit je wachtwoord wordt gemaakt. Zonder het wachtwoord is het bestand onleesbaar — voor iedereen, ook voor ons.' },
      {
        list: [
          'Bewaar het bestand in je map Documenten en een kopie op een usb-stick of in een cloudmap die je al gebruikt. De kopie blijft versleuteld.',
          'Je wachtwoord kan niet worden hersteld. Schrijf het op een veilige plek op, of gebruik een wachtwoordmanager.',
          'Niet-opgeslagen wijzigingen worden versleuteld in deze browser bewaard, zodat je ze bij verversen niet kwijtraakt. Als je je browsergegevens wist, verdwijnt die kopie — sla het bestand op om je werk te bewaren.',
          'Het wachtwoord wijzigen (Instellingen) geldt alleen voor kopieën die je daarna opslaat. Oudere kopieën openen nog met het oude wachtwoord, dus vervang ze.',
        ],
      },
    ],
  },
  {
    id: 'example',
    title: 'Probeer eerst de voorbeeldportefeuille',
    blocks: [
      { p: 'Het voorbeeld is een verzonnen belegger met indexfondsen, aandelen, een obligatiefonds, goud op de beurs en als munten, crypto, een spaarrekening, een belang in een start-up en doelen. Elk scherm werkt, dus je ziet wat de app doet voordat je iets importeert.' },
      { list: ['Open het vanaf het startscherm met “Probeer de voorbeeldportefeuille”, of ga naar het adres dat eindigt op #example.', 'Verander wat je wilt: er wordt niets opgeslagen, en de kopie van je eigen bestand in deze browser blijft onaangeroerd.', '“Voorbeeld sluiten” brengt je terug naar het startscherm.'] },
    ],
  },
  {
    id: 'bank',
    title: 'Bankrekeningen',
    blocks: [
      { p: 'Voor spaar- en betaalrekeningen is er geen afschrift om te importeren, dus je vult af en toe het saldo in. Het geld telt als contant geld: het vult je buffer en is zichtbaar in je vermogen en prognoses.' },
      { steps: ['Kies in het Overzicht “Bankrekening toevoegen” en geef hem een naam, zoals “ING Spaarrekening”, met de valuta en het saldo van vandaag.', 'Verandert het saldo, kies dan “Saldo bijwerken” en vul het hele nieuwe saldo in. De boekhouding legt alleen het verschil vast, als geld erbij of eraf.', 'Geef voor een rekening in een andere valuta de wisselkoers van die dag op.'] },
    ],
  },
  {
    id: 'metals',
    title: 'Fysiek goud en zilver',
    blocks: [
      { p: 'Munten en baren die je zelf bewaart, worden geteld in grammen zuiver metaal, zodat aankoopwaarde, waarde en winst werken zoals bij aandelen. Een goudfonds dat op de beurs wordt verhandeld, zoals Xetra-Gold, is een gewone positie: koop het op het scherm Handel.' },
      { steps: ['Kies in het Overzicht “Munten of baren toevoegen”. Vul het zuivere metaal in troy ounces of grammen in (een munt van één ounce bevat één ounce, ook als hij meer weegt) en alles wat je betaalde, inclusief de opslag van de handelaar.', 'Werk de prijs af en toe bij, per ounce of per gram. Tot die tijd wordt de prijs gebruikt die je betaalde.', 'Leg bij verkoop vast wat je ontving na de marge van de handelaar. De kosten gaan eerst af van je oudste aankopen.'] },
      { p: 'Op het scherm Belasting zie je bij elke verkoop wanneer het metaal is gekocht, omdat sommige landen goud belasten naar hoe lang je het hebt gehad. In Bulgarije behandelen de conceptregels het als belastingvrij; controleer dat bij een accountant.' },
    ],
  },
  {
    id: 'private',
    title: 'Privébeleggingen en hun uitbetaling',
    blocks: [
      { p: 'Geld in iets zonder marktprijs, zoals een co-investering bij je werkgever of een belang in een niet-beursgenoteerd bedrijf. Het telt mee in je vermogen tegen wat je betaalde, of tegen een waarde die je instelt, en blijft buiten de prognoses omdat je het niet kunt verkopen wanneer je wilt.' },
      { steps: ['Kies in het Overzicht “Privébelegging toevoegen”: wat je betaalde, wanneer, in welke valuta en tegen welke wisselkoers, en hoe het geld dat terugkomt wordt belast.', 'Stel een waarde in zodra je weet wat het waard is.', 'Komt het geld terug, kies dan “Uitbetaling vastleggen”: het bedrag vóór belasting, eventuele belasting die je werkgever inhield, en of het belast werd als loon of als vermogenswinst.'] },
      { p: 'De positie wordt dan gesloten. Wat je inlegde, krijg je terug, en het verschil wordt geboekt als inkomen of als winst. Loon heeft een eigen onderdeel op het scherm Belasting; een vermogenswinst staat bij je andere verkopen. Ging het geld naar een bankrekening die je hier bijhoudt, werk dan ook het saldo daarvan bij. “Uitbetaling terugdraaien” haalt hem weer weg.' },
    ],
  },
  {
    id: 'claude',
    title: 'Vraag het een AI-assistent',
    blocks: [
      { p: 'Een AI-assistent kan je portefeuille lezen, vragen beantwoorden met de cijfers van de app, je helpen plannen en vastleggen wat je besluit zodra je het goedkeurt. Het is optioneel: alles in Investor werkt ook zonder.' },
      { p: 'Druk in de desktopapp op de knop ✦ rechtsonder en kies een model: een op je computer via Ollama of LM Studio (er verlaat niets je computer), of Anthropic of OpenAI met je eigen API-sleutel. Elke wijziging die hij wil maken, toont een kaart om goed te keuren en wordt een niet-opgeslagen wijziging die je zoals altijd opslaat. Op de website legt de knop uit waarom de desktopapp nodig is: je portefeuille hoort niet via een webpagina naar een model te gaan.' },
      { p: 'Je kunt ook je eigen AI-tool gebruiken, zoals Claude Code, Cursor of VS Code. Open in de desktopapp de assistent, druk bovenaan op de stekkerknop en kies Aanzetten. Kopieer de getoonde opdracht naar Claude Code, of het adres naar de MCP-instellingen van een andere tool. Meer is het niet: niets installeren, geen wachtwoord om op te slaan.' },
      { list: ['Het werkt zolang de app open is en je bestand ontgrendeld. Vergrendel je het bestand of sluit je de app, dan kan je tool het niet meer lezen.', 'Elke wijziging die je tool wil maken, toont hier een kaart om goed te keuren en wordt een niet-opgeslagen wijziging die je zoals altijd opslaat.', 'Het adres bevat een geheim. Maak een nieuw adres als je het per ongeluk hebt gedeeld. Met een cloudmodel gaat wat je tool leest naar die aanbieder.', 'In Claude Code voegt /plugin install investor --marketplace dobrinov/investor kant-en-klare skills toe: een maandelijkse check-up, een transactie oefenen, belastingseizoen en pensioen.'] },
      { p: 'Dingen om te vragen:' },
      { list: ['“Wat heb ik in bezit, en hoeveel staat er contant?”', '“Vergelijk € 10.000 in één keer beleggen met spreiden over 12 maanden, over 20 jaar.”', '“Ik heb vandaag 2 VUAA op Xetra verkocht tegen 131,20. Leg het vast en vertel me de belasting.”', '“Lig ik op koers voor mijn doelen? Wat is er nodig om op mijn 60e met pensioen te gaan?”'] },
    ],
  },
  {
    id: 'export-ibkr',
    title: 'Exporteren uit Interactive Brokers',
    blocks: [
      { p: 'Investor leest een Activity Flex Query in CSV. Je stelt hem één keer in en draait hem wanneer je actuele gegevens wilt.' },
      {
        steps: [
          'Open in de IBKR Portal Performance & Reports → Flex Queries.',
          'Druk naast Activity Flex Query op + en noem hem “investor-export”.',
          'Vink deze onderdelen aan en druk in elk op Select All: Account Information, Trades (Executions), Cash Transactions, Transaction Fees, Open Positions (Summary), Corporate Actions, Transfers, Financial Instrument Information, Conversion Rates.',
          'Delivery configuration: Format CSV; Include header and trailer records — No; Include column headers — Yes; Include section code and line descriptor — Yes; Date format yyyy-MM-dd; Time format HH:mm:ss; Date/time separator “;”.',
          'Sla op en druk op Run. Kies Custom Date Range van je eerste storting tot vandaag. Beperkt IBKR de periode tot 365 dagen, draai hem dan één keer per jaar.',
          'Importeer het gedownloade bestand hier. Later opnieuw draaien kan veilig: al geïmporteerde regels worden overgeslagen.',
        ],
      },
      { p: 'Exporteer je alleen open posities, dan komen ze binnen als beginsaldo zonder aankoopdatums of kosten. Importeer je later de volledige geschiedenis, dan worden ze automatisch vervangen.' },
    ],
  },
  {
    id: 'export-revolut',
    title: 'Exporteren uit Revolut',
    blocks: [
      {
        steps: [
          'Open in de Revolut-app Invest (of Stocks).',
          'Open ⋯ of Documents → Brokerage account → Account statement.',
          'Kies Excel/CSV en de volledige periode, en importeer het bestand hier.',
        ],
      },
      { p: 'Afschriften van Revolut vermelden niet op welke beurs een transactie plaatsvond. In Bulgarije bepaalt dat of een winst belastingvrij is, dus zulke verkopen krijgen op het scherm Belasting de markering “check handelsplaats”.' },
    ],
  },
  {
    id: 'add-broker',
    title: 'Ontbreekt je broker of beurs?',
    blocks: [
      { p: 'Elke broker wordt beschreven door een klein configuratiebestand, dus een nieuwe toevoegen vraagt geen programmeerwerk. Er zijn twee manieren.' },
      { p: 'Vraag erom. Open een issue op GitHub met de naam van de broker en een geanonimiseerde export:' },
      {
        steps: [
          'Exporteer een afschrift met in elk geval een storting, een aankoop, een verkoop, een dividend en kosten, als je die hebt.',
          'Vervang je rekeningnummer, naam en bedragen door verzonnen waarden. Houd de kolomnamen, het datumformaat en de typelabels precies zoals ze zijn geëxporteerd — die leest de configuratie.',
          'Open een nieuw issue op GitHub en voeg het bestand toe.',
        ],
      },
      { link: 'Open een issue op GitHub', href: 'https://github.com/dobrinov/investor/issues/new' },
      { p: 'Voeg hem zelf toe. Schrijf een configuratiebestand zoals dit, zet het in de map brokers/ van het project en open een pull request. Regels die je configuratie niet herkent, worden tijdens het importeren met regelnummer getoond, zodat je ziet wat er nog ontbreekt.' },
      { code: brokerExample },
      { p: 'Soorten regels: buy, sell, deposit, withdrawal, cash_transfer, dividend, withholding_tax, fee, interest, fx_pair_trade, position_snapshot, ignore. Soorten kosten: commission, fx_conversion, exchange, transaction_tax, custody, interest, other.' },
    ],
  },
  {
    id: 'add-tax',
    title: 'Ontbreekt je belastingdienst?',
    blocks: [
      { p: 'Belastingregels zitten in pakketten: één bestand per land en belastingjaar, in de map jurisdictions/ van het project (bijvoorbeeld jurisdictions/bg/2026.toml). Een pakket legt de tarieven, vrijstellingen, formulieren en termijnen vast.' },
      {
        steps: [
          'Open een issue op GitHub met het land en het belastingjaar, en link naar de officiële uitleg van de belastingdienst.',
          'Of kopieer het Bulgaarse pakket, pas de waarden aan en open een pull request.',
          'Laat verified = false staan tot een belastingadviseur het pakket heeft gecontroleerd. De app toont een waarschuwing bij niet-gecontroleerde pakketten.',
        ],
      },
      { code: packExample },
      { p: 'De waarden hierboven zijn plaatsvervangers. Elk tarief, elke vrijstelling en elke formuliernaam moet uit de officiële bronnen komen.' },
      { link: 'Open een issue op GitHub', href: 'https://github.com/dobrinov/investor/issues/new' },
    ],
  },
  {
    id: 'glossary',
    title: 'Begrippen in de app',
    blocks: [
      {
        terms: [
          ['Dubbel boekhouden', 'Elke geldbeweging wordt twee keer vastgelegd — waar het vandaan kwam en waar het naartoe ging — zodat de boekhouding altijd sluit.'],
          ['Journaalpost', 'Eén gebeurtenis, zoals een aankoop of een dividend, vastgelegd als boekingsregels die samen nul zijn.'],
          ['Proefbalans', 'Het saldo van elke rekening. Debet is gelijk aan credit als de boekhouding klopt.'],
          ['Aankoopwaarde', 'Wat je voor een belegging betaalde, inclusief kosten waar de regels dat toestaan.'],
          ['FIFO', 'First in, first out: als je verkoopt, gelden de oudste stuks die je kocht als eerst verkocht.'],
          ['Gerealiseerde winst', 'Winst of verlies dat vastligt doordat je verkocht. Hierover wordt belasting geheven.'],
          ['Ongerealiseerde winst', 'Winst of verlies op beleggingen die je nog hebt, tegen hun laatste koers.'],
          ['TER', 'Total expense ratio: de jaarlijkse lopende kosten van een fonds, verrekend in de koers.'],
          ['Bronbelasting', 'Belasting die een ander land inhoudt voordat een dividend bij jou aankomt. Vaak deels verrekenbaar in je eigen land.'],
          ['Handelsplaats', 'De beurs waar een transactie plaatsvond, zoals Xetra of NASDAQ.'],
          ['ETF', 'Een fonds dat als een aandeel op de beurs wordt verhandeld en meestal in veel bedrijven tegelijk belegt.'],
          ['Buffer', 'Geld dat je contant aanhoudt voor onverwachte uitgaven, zodat je nooit op een slecht moment beleggingen hoeft te verkopen.'],
        ],
      },
    ],
  },
]
