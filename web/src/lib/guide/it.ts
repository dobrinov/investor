import { brokerExample, packExample } from './en'
import type { Section } from './types'

export const it: Section[] = [
  {
    id: 'start',
    title: 'Primi passi',
    blocks: [
      { p: 'Investor conserva la tua contabilità, i tuoi piani e i tuoi dati fiscali in un unico file cifrato sul tuo computer. Una buona prima sessione:' },
      {
        steps: [
          'Esporta gli estratti conto da ogni broker (vedi le sezioni seguenti).',
          'Importali nella schermata Importa estratti conto. Ogni operazione, dividendo, commissione e imposta diventa una scrittura contabile in pareggio.',
          'Controlla la Panoramica. Le posizioni senza prezzo sono contate a quanto le hai pagate; aggiungi un prezzo con «Aggiorna prezzo».',
          'Compila il profilo e aggiungi gli obiettivi nella schermata Obiettivi.',
          'Esplora le Proiezioni, e Tasse e dichiarazioni prima di presentare la dichiarazione.',
          'Salva il file .invest e conservane una copia di backup.',
        ],
      },
    ],
  },
  {
    id: 'file',
    title: 'Il tuo file, la tua password',
    blocks: [
      { p: 'Non c’è un account e non c’è un server. Tutto vive nel tuo file .invest, cifrato con una chiave ricavata dalla tua password. Senza la password il file è illeggibile per chiunque, noi compresi.' },
      {
        list: [
          'Tieni il file nella cartella Documenti e una copia su una chiavetta USB o in una cartella cloud che usi già. La copia resta cifrata.',
          'Non è possibile reimpostare la password. Annotala in un posto sicuro o usa un gestore di password.',
          'Le modifiche non salvate sono conservate cifrate in questo browser, così un aggiornamento della pagina non le fa perdere. Se cancelli i dati del browser, quella copia sparisce: salva il file per non perdere il lavoro.',
          'Cambiare la password (Impostazioni) vale solo per le copie salvate dopo. Le copie più vecchie si aprono ancora con la vecchia password, quindi sostituiscile.',
        ],
      },
    ],
  },
  {
    id: 'example',
    title: 'Prova prima il portafoglio di esempio',
    blocks: [
      { p: 'L’esempio è un investitore inventato con fondi indicizzati, azioni, un fondo obbligazionario, oro in borsa e in monete, criptovalute, un conto deposito, una partecipazione in una startup e degli obiettivi. Tutte le schermate funzionano, così puoi vedere cosa fa l’app prima di importare qualsiasi cosa.' },
      { list: ['Aprilo dalla schermata iniziale con «Prova il portafoglio di esempio», oppure vai all’indirizzo che termina con #example.', 'Modifica quello che vuoi: non viene salvato nulla, e la copia del tuo file conservata in questo browser non viene toccata.', '«Chiudi l’esempio» ti riporta alla schermata iniziale.'] },
    ],
  },
  {
    id: 'bank',
    title: 'Conti bancari',
    blocks: [
      { p: 'I conti deposito e i conti correnti non hanno un estratto conto da importare, quindi inserisci il saldo ogni tanto. Il denaro conta come liquidità: alimenta il fondo di emergenza e compare nel patrimonio netto e nelle proiezioni.' },
      { steps: ['In Panoramica, scegli «Aggiungi un conto bancario» e dagli un nome, ad esempio «Risparmi UBB», con la valuta e il saldo di oggi.', 'Ogni volta che il saldo cambia, scegli «Aggiorna saldo» e scrivi il nuovo saldo intero. La contabilità registra solo la differenza, come denaro versato o prelevato.', 'Per un conto in un’altra valuta, indica il cambio di quel giorno.'] },
    ],
  },
  {
    id: 'metals',
    title: 'Oro e argento fisici',
    blocks: [
      { p: 'Le monete e i lingotti che custodisci tu sono contati in grammi di metallo puro, quindi costo, valore e plusvalenze funzionano come per le azioni. Un fondo sull’oro quotato in borsa, come Xetra-Gold, è invece una posizione normale: compralo nella schermata Opera.' },
      { steps: ['In Panoramica, scegli «Aggiungi monete o lingotti». Inserisci il metallo puro in once troy o in grammi (una moneta da un’oncia contiene un’oncia anche se pesa di più) e tutto ciò che hai pagato, compreso il premio del commerciante.', 'Aggiorna il prezzo ogni tanto, per oncia o per grammo. Finché non lo fai, si usa il prezzo che hai pagato.', 'Quando vendi, registra quanto hai ricevuto al netto del margine del commerciante. Il costo viene scaricato prima dagli acquisti più vecchi.'] },
      { p: 'Nella schermata Tasse ogni vendita mostra quando è stato acquistato il metallo, perché diversi paesi tassano l’oro in base a quanto a lungo lo hai tenuto. In Bulgaria le regole in bozza lo considerano esente; verificalo con un commercialista.' },
    ],
  },
  {
    id: 'private',
    title: 'Investimenti privati e il loro incasso',
    blocks: [
      { p: 'Denaro in qualcosa senza prezzo di mercato, come un coinvestimento con il datore di lavoro o una partecipazione in una società non quotata. Conta nel patrimonio netto a quanto hai pagato, o al valore che imposti, e resta fuori dalle proiezioni perché non puoi venderlo quando vuoi.' },
      { steps: ['In Panoramica, scegli «Aggiungi un investimento privato»: quanto hai pagato, quando, in quale valuta e a quale cambio, e come sarà tassato il denaro che rientra.', 'Imposta un valore ogni volta che scopri quanto vale.', 'Quando il denaro rientra, scegli «Registra incasso»: l’importo lordo, l’eventuale ritenuta del datore di lavoro e se è stato tassato come reddito da lavoro dipendente o come plusvalenza.'] },
      { p: 'A quel punto l’investimento si chiude. Ciò che hai versato ti torna indietro e la differenza viene registrata come reddito o come plusvalenza. Il reddito da lavoro ha una sua sezione nella schermata Tasse; una plusvalenza compare insieme alle altre vendite. Se il denaro è arrivato su un conto bancario che segui qui, aggiorna anche il suo saldo. «Annulla incasso» lo toglie di nuovo.' },
    ],
  },
  {
    id: 'claude',
    title: 'Chiedi a un assistente IA',
    blocks: [
      { p: 'Un assistente IA può leggere il tuo portafoglio, rispondere alle domande con i numeri dell’app, aiutarti a pianificare e registrare ciò che decidi dopo che lo hai approvato. È facoltativo: tutto in Investor funziona anche senza.' },
      { p: 'Nell’app desktop, premi il pulsante ✦ nell’angolo in basso a destra e scegli un modello: uno sul tuo computer tramite Ollama o LM Studio (nulla lascia il tuo computer), oppure Anthropic o OpenAI con la tua chiave API. Ogni modifica che vuole fare mostra una scheda da approvare e diventa una modifica non salvata che salvi come al solito. Sul sito, il pulsante spiega perché serve l’app desktop: il tuo portafoglio non dovrebbe passare da una pagina web per arrivare a un modello.' },
      { p: 'Puoi anche usare il tuo strumento di IA, come Claude Code, Cursor o VS Code. Nell’app desktop, apri l’assistente, premi il pulsante con la spina in alto e scegli Attiva. Copia il comando mostrato in Claude Code, oppure l’indirizzo nelle impostazioni MCP di un altro strumento. Tutto qui: niente da installare, nessuna password da conservare.' },
      { list: ['Funziona finché l’app è aperta e il file è sbloccato. Se blocchi il file o chiudi l’app, il tuo strumento non può più leggerlo.', 'Ogni modifica che il tuo strumento vuole fare mostra qui una scheda da approvare e diventa una modifica non salvata che salvi come al solito.', 'L’indirizzo contiene un segreto. Creane uno nuovo se l’hai condiviso per errore. Con un modello cloud, ciò che il tuo strumento legge viene inviato a quel fornitore.', 'In Claude Code, /plugin install investor --marketplace dobrinov/investor aggiunge skill pronte all’uso: un controllo mensile, la prova di un’operazione, la stagione delle tasse e la pensione.'] },
      { p: 'Cose da chiedere:' },
      { list: ['«Cosa possiedo e quanto ho in liquidità?»', '«Confronta l’investimento di 10.000 € in un’unica soluzione con la distribuzione su 12 mesi, su 20 anni.»', '«Oggi ho venduto 2 VUAA su Xetra a 131,20. Registralo e dimmi l’imposta.»', '«Sono in linea con i miei obiettivi? Cosa servirebbe per andare in pensione a 60 anni?»'] },
    ],
  },
  {
    id: 'export-ibkr',
    title: 'Esportare da Interactive Brokers',
    blocks: [
      { p: 'Investor legge una Activity Flex Query in CSV. La configuri una volta e la esegui ogni volta che vuoi dati aggiornati.' },
      {
        steps: [
          'Nell’IBKR Portal apri Performance & Reports → Flex Queries.',
          'Accanto ad Activity Flex Query premi + e chiamala «investor-export».',
          'Spunta queste sezioni e premi Select All in ognuna: Account Information, Trades (Executions), Cash Transactions, Transaction Fees, Open Positions (Summary), Corporate Actions, Transfers, Financial Instrument Information, Conversion Rates.',
          'Delivery configuration: Format CSV; Include header and trailer records — No; Include column headers — Yes; Include section code and line descriptor — Yes; Date format yyyy-MM-dd; Time format HH:mm:ss; Date/time separator «;».',
          'Salva, poi premi Run. Scegli Custom Date Range dal primo versamento a oggi. Se IBKR limita il periodo a 365 giorni, eseguila una volta per ogni anno.',
          'Importa qui il file scaricato. Eseguirla di nuovo in seguito è sicuro: le righe già importate vengono saltate.',
        ],
      },
      { p: 'Se esporti solo le posizioni aperte, arrivano come saldi di apertura senza date di acquisto né commissioni. Importando più tardi lo storico completo, vengono sostituite automaticamente.' },
    ],
  },
  {
    id: 'export-revolut',
    title: 'Esportare da Revolut',
    blocks: [
      {
        steps: [
          'Nell’app Revolut apri Invest (o Stocks).',
          'Apri ⋯ oppure Documents → Brokerage account → Account statement.',
          'Scegli Excel/CSV e l’intero periodo, poi importa qui il file.',
        ],
      },
      { p: 'Gli estratti conto di Revolut non indicano su quale mercato è avvenuta un’operazione. In Bulgaria questo decide se una plusvalenza è esente, quindi queste vendite sono segnalate con «Verifica il mercato» nella schermata Tasse.' },
    ],
  },
  {
    id: 'add-broker',
    title: 'Manca il tuo broker o il tuo mercato?',
    blocks: [
      { p: 'Ogni broker è descritto da un piccolo file di configurazione, quindi aggiungerne uno non richiede di programmare. Ci sono due modi.' },
      { p: 'Chiedilo. Apri una issue su GitHub con il nome del broker e un’esportazione anonimizzata:' },
      {
        steps: [
          'Esporta un estratto conto che contenga almeno un versamento, un acquisto, una vendita, un dividendo e una commissione, se li hai.',
          'Sostituisci numero di conto, nome e importi con valori inventati. Lascia i nomi delle colonne, il formato delle date e le etichette dei tipi esattamente come esportati: sono ciò che legge la configurazione.',
          'Apri una nuova issue su GitHub e allega il file.',
        ],
      },
      { link: 'Apri una issue su GitHub', href: 'https://github.com/dobrinov/investor/issues/new' },
      { p: 'Aggiungilo tu. Scrivi un file di configurazione come questo, mettilo nella cartella brokers/ del progetto e apri una pull request. Durante l’importazione, le righe che la configurazione non riconosce vengono elencate con il numero di riga, così vedi cosa manca ancora.' },
      { code: brokerExample },
      { p: 'Tipi di righe: buy, sell, deposit, withdrawal, cash_transfer, dividend, withholding_tax, fee, interest, fx_pair_trade, position_snapshot, ignore. Tipi di commissioni: commission, fx_conversion, exchange, transaction_tax, custody, interest, other.' },
    ],
  },
  {
    id: 'add-tax',
    title: 'Manca la tua amministrazione fiscale?',
    blocks: [
      { p: 'Le regole fiscali sono raccolte in pacchetti: un file per paese e anno d’imposta, nella cartella jurisdictions/ del progetto (ad esempio jurisdictions/bg/2026.toml). Un pacchetto stabilisce aliquote, esenzioni, modelli e scadenze.' },
      {
        steps: [
          'Apri una issue su GitHub indicando il paese e l’anno d’imposta, con il link alle indicazioni ufficiali dell’amministrazione fiscale.',
          'Oppure copia il pacchetto bulgaro, cambia i valori e apri una pull request.',
          'Lascia verified = false finché un consulente fiscale non ha controllato il pacchetto. L’app mostra un avviso per i pacchetti non verificati.',
        ],
      },
      { code: packExample },
      { p: 'I valori qui sopra sono segnaposto. Ogni aliquota, esenzione e nome di modello deve provenire dalle fonti ufficiali.' },
      { link: 'Apri una issue su GitHub', href: 'https://github.com/dobrinov/investor/issues/new' },
    ],
  },
  {
    id: 'glossary',
    title: 'Termini usati nell’app',
    blocks: [
      {
        terms: [
          ['Partita doppia', 'Ogni movimento di denaro viene registrato due volte, da dove arriva e dove va, così la contabilità è sempre in pareggio.'],
          ['Scrittura contabile', 'Un singolo evento, come un acquisto o un dividendo, registrato in movimenti che si sommano a zero.'],
          ['Bilancio di verifica', 'Il saldo di ogni conto. Se la contabilità è corretta, il totale dare è uguale al totale avere.'],
          ['Costo di carico', 'Quanto hai pagato per un investimento, comprese le commissioni dove le regole lo consentono.'],
          ['FIFO', 'First in, first out: quando vendi, si considerano vendute per prime le quote acquistate per prime.'],
          ['Plusvalenza realizzata', 'Utile o perdita consolidati con la vendita. È ciò che viene tassato.'],
          ['Utile non realizzato', 'Utile o perdita sugli investimenti che possiedi ancora, all’ultimo prezzo.'],
          ['TER', 'Total expense ratio: il costo annuo di gestione di un fondo, prelevato dal suo prezzo.'],
          ['Ritenuta alla fonte', 'Imposta trattenuta da un altro paese prima che un dividendo ti arrivi. Spesso è in parte recuperabile come credito d’imposta nel tuo paese.'],
          ['Sede di negoziazione', 'Il mercato su cui è avvenuta un’operazione, come Xetra o NASDAQ.'],
          ['ETF', 'Un fondo quotato in borsa come un’azione, che di solito contiene molte società insieme.'],
          ['Fondo di emergenza', 'Denaro tenuto in liquidità per gli imprevisti, così non devi mai vendere investimenti nel momento sbagliato.'],
        ],
      },
    ],
  },
]
