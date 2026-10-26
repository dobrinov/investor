import { brokerExample, packExample } from './en'
import type { Section } from './types'

export const ro: Section[] = [
  {
    id: 'start',
    title: 'Primii pași',
    blocks: [
      { p: 'Investor îți păstrează evidența, planurile și cifrele fiscale într-un singur fișier criptat, pe calculatorul tău. Un început bun:' },
      {
        steps: [
          'Exportă extrasele de la fiecare broker (vezi secțiunile următoare).',
          'Importă-le în ecranul „Importă extrase”. Fiecare tranzacție, dividend, comision și impozit devine o înregistrare contabilă echilibrată.',
          'Verifică „Rezumat”. Deținerile fără preț sunt luate la cât ai plătit; adaugă un preț cu „Actualizează prețul”.',
          'Completează-ți profilul și adaugă obiective în ecranul „Obiective”.',
          'Explorează „Proiecții”, iar înainte de a depune declarația, „Impozite și raportare”.',
          'Salvează fișierul .invest și păstrează o copie de rezervă.',
        ],
      },
    ],
  },
  {
    id: 'file',
    title: 'Fișierul tău, parola ta',
    blocks: [
      { p: 'Nu există cont și nu există server. Totul stă în fișierul tău .invest, criptat cu o cheie obținută din parola ta. Fără parolă, fișierul nu poate fi citit — de nimeni, nici de noi.' },
      {
        list: [
          'Păstrează fișierul în folderul Documente și o copie pe un stick USB sau într-un folder în cloud pe care îl folosești deja. Copia rămâne criptată.',
          'Parola nu poate fi resetată. Noteaz-o într-un loc sigur sau folosește un manager de parole.',
          'Modificările nesalvate sunt păstrate criptat în acest browser, ca să nu se piardă la reîncărcarea paginii. Dacă ștergi datele browserului, ștergi și această copie — salvează fișierul ca să nu-ți pierzi munca.',
          'Schimbarea parolei („Setări”) afectează doar copiile salvate după aceea. Copiile mai vechi se deschid în continuare cu parola veche, așa că înlocuiește-le.',
        ],
      },
    ],
  },
  {
    id: 'example',
    title: 'Încearcă întâi portofoliul exemplu',
    blocks: [
      { p: 'Exemplul este un investitor inventat, cu fonduri index, acțiuni, un fond de obligațiuni, aur la bursă și sub formă de monede, criptomonede, un cont de economii, o participație într-un startup și obiective. Toate ecranele funcționează, așa că poți vedea ce face aplicația înainte să imporți ceva.' },
      { list: ['Deschide-l din ecranul de start cu „Încearcă portofoliul exemplu” sau mergi la adresa care se termină cu #example.', 'Schimbă orice vrei: nimic nu se salvează, iar copia propriului tău fișier din acest browser rămâne neatinsă.', '„Închide exemplul” te duce înapoi la ecranul de start.'] },
    ],
  },
  {
    id: 'bank',
    title: 'Conturi bancare',
    blocks: [
      { p: 'Conturile de economii și conturile curente nu au extras de importat, așa că introduci soldul din când în când. Banii contează ca numerar: alimentează fondul de urgență și apar în averea netă și în proiecții.' },
      { steps: ['În „Rezumat”, alege „Adaugă un cont bancar” și dă-i un nume, de exemplu „UBB economii”, împreună cu moneda și soldul de azi.', 'Ori de câte ori se schimbă soldul, alege „Actualizează soldul” și scrie noul sold complet. Evidența înregistrează doar diferența, ca bani depuși sau retrași.', 'Pentru un cont în altă monedă, introdu cursul de schimb din ziua respectivă.'] },
    ],
  },
  {
    id: 'metals',
    title: 'Aur și argint fizic',
    blocks: [
      { p: 'Monedele și lingourile pe care le păstrezi singur sunt socotite în grame de metal pur, așa că costul, valoarea și câștigurile funcționează ca la acțiuni. Un fond de aur tranzacționat la bursă, cum e Xetra-Gold, este o deținere obișnuită: cumpără-l din ecranul „Tranzacție”.' },
      { steps: ['În „Rezumat”, alege „Adaugă monede sau lingouri”. Introdu metalul pur în uncii troy sau grame (o monedă de o uncie conține o uncie, chiar dacă cântărește mai mult) și tot ce ai plătit, inclusiv prima dealerului.', 'Actualizează prețul din când în când, pe uncie sau pe gram. Până atunci se folosește prețul plătit.', 'Când vinzi, înregistrează ce ai primit după marja dealerului. Costul se scade întâi din cele mai vechi achiziții.'] },
      { p: 'În ecranul Impozite, la fiecare vânzare se vede când a fost cumpărat metalul, pentru că mai multe țări impozitează aurul în funcție de cât timp l-ai deținut. În Bulgaria, regulile preliminare îl tratează ca neimpozabil; verifică asta cu un contabil.' },
    ],
  },
  {
    id: 'private',
    title: 'Dețineri private și plata lor',
    blocks: [
      { p: 'Bani puși în ceva fără preț de piață, cum ar fi o coinvestiție cu angajatorul sau o participație într-o companie privată. Contează în averea netă la cât ai plătit sau la o valoare pe care o setezi și nu intră în proiecții, pentru că nu le poți vinde când vrei.' },
      { steps: ['În „Rezumat”, alege „Adaugă o deținere privată”: cât ai plătit, când, în ce monedă și la ce curs, și cum vor fi impozitați banii care se întorc.', 'Setează o valoare ori de câte ori afli cât valorează.', 'Când banii se întorc, alege „Înregistrează plata”: suma înainte de impozit, impozitul reținut de angajator și dacă s-a impozitat ca venit salarial sau ca câștig de capital.'] },
      { p: 'Apoi deținerea se închide. Ce ai investit îți revine, iar diferența se înregistrează ca venit sau ca câștig. Venitul salarial are propria secțiune în ecranul Impozite; un câștig de capital apare alături de celelalte vânzări. Dacă banii au ajuns într-un cont bancar pe care îl urmărești aici, actualizează-i și soldul. „Anulează plata” o scoate din nou.' },
    ],
  },
  {
    id: 'claude',
    title: 'Întreabă un asistent AI',
    blocks: [
      { p: 'Un asistent AI îți poate citi portofoliul, poate răspunde la întrebări cu cifrele aplicației, te poate ajuta să planifici și poate înregistra ce decizi, după ce aprobi. Este opțional: totul în Investor funcționează și fără el.' },
      { p: 'În aplicația desktop, apasă butonul ✦ din colțul din dreapta jos și alege un model: unul de pe calculatorul tău, prin Ollama sau LM Studio (nimic nu părăsește calculatorul), sau Anthropic ori OpenAI, cu propria cheie API. Fiecare modificare pe care vrea s-o facă afișează un card de aprobat și devine o modificare nesalvată, pe care o salvezi ca de obicei. Pe site, butonul explică de ce e nevoie de aplicația desktop: portofoliul tău n-ar trebui să treacă printr-o pagină web în drum spre un model.' },
      { p: 'Poți folosi și propriul instrument AI, cum ar fi Claude Code, Cursor sau VS Code. În aplicația desktop, deschide asistentul, apasă butonul cu ștecher din partea de sus și alege „Activează”. Copiază comanda afișată în Claude Code sau adresa în setările MCP ale altui instrument. Asta e tot: nimic de instalat, nicio parolă de păstrat.' },
      { list: ['Funcționează cât timp aplicația e deschisă și fișierul deblocat. Dacă blochezi fișierul sau închizi aplicația, instrumentul tău nu-l mai poate citi.', 'Fiecare modificare pe care instrumentul vrea s-o facă afișează aici un card de aprobat și devine o modificare nesalvată, pe care o salvezi ca de obicei.', 'Adresa conține un secret. Creează o adresă nouă dacă ai partajat-o din greșeală. Cu un model în cloud, ce citește instrumentul tău este trimis acelui furnizor.', 'În Claude Code, /plugin install investor --marketplace dobrinov/investor adaugă abilități gata făcute: o verificare lunară, testarea unei tranzacții, sezonul fiscal și pensionarea.'] },
      { p: 'Ce poți întreba:' },
      { list: ['„Ce dețin și cât am în numerar?”', '„Compară investirea a 10.000 € deodată cu eșalonarea pe 12 luni, pe un orizont de 20 de ani.”', '„Am vândut azi 2 VUAA pe Xetra la 131,20. Înregistrează și spune-mi impozitul.”', '„Sunt pe drumul cel bun cu obiectivele? Ce mi-ar trebui ca să mă pensionez la 60 de ani?”'] },
    ],
  },
  {
    id: 'export-ibkr',
    title: 'Export din Interactive Brokers',
    blocks: [
      { p: 'Investor citește un Activity Flex Query în format CSV. Îl configurezi o dată și îl rulezi ori de câte ori vrei date proaspete.' },
      {
        steps: [
          'În IBKR Portal, deschide Performance & Reports → Flex Queries.',
          'Lângă Activity Flex Query apasă + și numește-l „investor-export”.',
          'Bifează aceste secțiuni și apasă Select All în fiecare: Account Information, Trades (Executions), Cash Transactions, Transaction Fees, Open Positions (Summary), Corporate Actions, Transfers, Financial Instrument Information, Conversion Rates.',
          'Delivery configuration: Format CSV; Include header and trailer records — No; Include column headers — Yes; Include section code and line descriptor — Yes; Date format yyyy-MM-dd; Time format HH:mm:ss; Date/time separator „;”.',
          'Salvează, apoi apasă Run. Alege Custom Date Range de la primul depozit până azi. Dacă IBKR limitează perioada la 365 de zile, rulează-l o dată pentru fiecare an.',
          'Importă aici fișierul descărcat. Îl poți rula din nou mai târziu fără grijă: rândurile deja importate sunt omise.',
        ],
      },
      { p: 'Dacă exporți doar pozițiile deschise, deținerile apar ca solduri inițiale, fără date de cumpărare sau comisioane. Dacă imporți mai târziu tot istoricul, acestea sunt înlocuite automat.' },
    ],
  },
  {
    id: 'export-revolut',
    title: 'Export din Revolut',
    blocks: [
      {
        steps: [
          'În aplicația Revolut, deschide Invest (sau Stocks).',
          'Deschide ⋯ sau Documents → Brokerage account → Account statement.',
          'Alege Excel/CSV și toată perioada, apoi importă fișierul aici.',
        ],
      },
      { p: 'Extrasele Revolut nu spun pe ce bursă a avut loc tranzacția. În Bulgaria, de asta depinde dacă un câștig e scutit de impozit, așa că aceste vânzări sunt marcate cu „Verifică locul” în ecranul Impozite.' },
    ],
  },
  {
    id: 'add-broker',
    title: 'Lipsește brokerul sau bursa ta?',
    blocks: [
      { p: 'Fiecare broker este descris de un mic fișier de configurație, așa că adăugarea unuia nu cere programare. Există două variante.' },
      { p: 'Cere-l. Deschide un issue pe GitHub cu numele brokerului și un export anonimizat:' },
      {
        steps: [
          'Exportă un extras care să conțină, dacă le ai, cel puțin un depozit, o cumpărare, o vânzare, un dividend și un comision.',
          'Înlocuiește numărul de cont, numele și sumele cu valori inventate. Păstrează numele coloanelor, formatul datelor și etichetele de tip exact cum au fost exportate — pe acestea le citește configurația.',
          'Deschide un issue nou pe GitHub și atașează fișierul.',
        ],
      },
      { link: 'Deschide un issue pe GitHub', href: 'https://github.com/dobrinov/investor/issues/new' },
      { p: 'Adaugă-l singur. Scrie un fișier de configurație ca acesta, pune-l în folderul brokers/ al proiectului și deschide un pull request. La import, rândurile pe care configurația nu le recunoaște sunt afișate cu numărul lor, așa că vezi ce mai lipsește.' },
      { code: brokerExample },
      { p: 'Tipuri de rânduri: buy, sell, deposit, withdrawal, cash_transfer, dividend, withholding_tax, fee, interest, fx_pair_trade, position_snapshot, ignore. Tipuri de comisioane: commission, fx_conversion, exchange, transaction_tax, custody, interest, other.' },
    ],
  },
  {
    id: 'add-tax',
    title: 'Lipsește autoritatea fiscală din țara ta?',
    blocks: [
      { p: 'Regulile fiscale vin în pachete: câte un fișier pentru fiecare țară și an fiscal, în folderul jurisdictions/ al proiectului (de exemplu jurisdictions/bg/2026.toml). Un pachet stabilește cotele, scutirile, formularele și termenele.' },
      {
        steps: [
          'Deschide un issue pe GitHub cu țara și anul fiscal și adaugă un link către ghidul oficial al autorității fiscale.',
          'Sau copiază pachetul bulgăresc, schimbă valorile și deschide un pull request.',
          'Lasă verified = false până când un consultant fiscal verifică pachetul. Aplicația afișează un avertisment pentru pachetele neverificate.',
        ],
      },
      { code: packExample },
      { p: 'Valorile de mai sus sunt doar exemple. Fiecare cotă, scutire și denumire de formular trebuie să provină din surse oficiale.' },
      { link: 'Deschide un issue pe GitHub', href: 'https://github.com/dobrinov/investor/issues/new' },
    ],
  },
  {
    id: 'glossary',
    title: 'Termeni folosiți în aplicație',
    blocks: [
      {
        terms: [
          ['Contabilitate în partidă dublă', 'Fiecare mișcare de bani este înregistrată de două ori — de unde vine și unde ajunge — așa că evidența este mereu echilibrată.'],
          ['Înregistrare contabilă', 'Un eveniment, de exemplu o cumpărare sau un dividend, înregistrat ca rânduri care însumează zero.'],
          ['Balanță de verificare', 'Soldul fiecărui cont. Când evidența e corectă, debitele sunt egale cu creditele.'],
          ['Cost de achiziție', 'Cât ai plătit pentru o investiție, inclusiv comisioanele, acolo unde regulile permit.'],
          ['FIFO', 'First in, first out („primul intrat, primul ieșit”): când vinzi, cele mai vechi unități cumpărate sunt considerate vândute primele.'],
          ['Câștig realizat', 'Profit sau pierdere fixate prin vânzare. Acesta se impozitează.'],
          ['Câștig nerealizat', 'Profit sau pierdere la investițiile pe care încă le deții, la ultimul lor preț.'],
          ['TER', 'Total expense ratio (rata totală a cheltuielilor): costul anual de funcționare al unui fond, scăzut din prețul lui.'],
          ['Impozit reținut la sursă', 'Impozit reținut de o altă țară înainte ca dividendul să ajungă la tine. Deseori se poate credita parțial în țara ta.'],
          ['Loc de tranzacționare', 'Bursa pe care a avut loc tranzacția, de exemplu Xetra sau NASDAQ.'],
          ['ETF', 'Un fond tranzacționat la bursă ca o acțiune, care de obicei deține multe companii deodată.'],
          ['Fond de urgență', 'Bani ținuți în numerar pentru neprevăzut, ca să nu fii nevoit niciodată să vinzi investiții într-un moment prost.'],
        ],
      },
    ],
  },
]
