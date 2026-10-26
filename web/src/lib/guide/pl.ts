import { brokerExample, packExample } from './en'
import type { Section } from './types'

export const pl: Section[] = [
  {
    id: 'start',
    title: 'Pierwsze kroki',
    blocks: [
      { p: 'Investor trzyma twoje księgi, plany i liczby podatkowe w jednym zaszyfrowanym pliku na twoim komputerze. Dobry plan na początek:' },
      {
        steps: [
          'Wyeksportuj wyciągi od każdego brokera (zobacz kolejne sekcje).',
          'Zaimportuj je na ekranie „Import wyciągów”. Każda transakcja, dywidenda, opłata i podatek staje się zbilansowanym zapisem księgowym.',
          'Sprawdź „Przegląd”. Pozycje bez ceny są liczone według tego, ile zapłaciłeś; dodaj cenę przyciskiem „Zaktualizuj cenę”.',
          'Uzupełnij profil i dodaj cele na ekranie „Cele”.',
          'Przejrzyj „Prognozy”, a przed złożeniem zeznania — „Podatki i rozliczenia”.',
          'Zapisz plik .invest i zrób kopię zapasową.',
        ],
      },
    ],
  },
  {
    id: 'file',
    title: 'Twój plik, twoje hasło',
    blocks: [
      { p: 'Nie ma konta ani serwera. Wszystko jest w pliku .invest, zaszyfrowanym kluczem utworzonym z twojego hasła. Bez hasła pliku nie da się odczytać — nikomu, łącznie z nami.' },
      {
        list: [
          'Trzymaj plik w folderze Dokumenty, a kopię na pendrivie albo w chmurze, z której już korzystasz. Kopia pozostaje zaszyfrowana.',
          'Hasła nie da się zresetować. Zapisz je w bezpiecznym miejscu albo użyj menedżera haseł.',
          'Niezapisane zmiany są przechowywane w tej przeglądarce w postaci zaszyfrowanej, więc odświeżenie strony ich nie usuwa. Wyczyszczenie danych przeglądarki kasuje tę kopię — zapisz plik, aby zachować swoją pracę.',
          'Zmiana hasła (Ustawienia) dotyczy tylko kopii zapisanych później. Starsze kopie nadal otwierają się starym hasłem, więc je zastąp.',
        ],
      },
    ],
  },
  {
    id: 'example',
    title: 'Najpierw wypróbuj przykładowy portfel',
    blocks: [
      { p: 'Przykład to zmyślony inwestor z funduszami indeksowymi, akcjami, funduszem obligacji, złotem na giełdzie i w monetach, kryptowalutami, kontem oszczędnościowym, udziałem w startupie i celami. Każdy ekran działa, więc możesz zobaczyć, co robi aplikacja, zanim cokolwiek zaimportujesz.' },
      { list: ['Otwórz go z ekranu startowego przyciskiem „Wypróbuj przykładowy portfel” albo wejdź na adres kończący się na #example.', 'Zmieniaj, co chcesz: nic nie jest zapisywane, a kopia twojego własnego pliku w tej przeglądarce pozostaje nietknięta.', '„Zamknij przykład” przenosi cię z powrotem na ekran startowy.'] },
    ],
  },
  {
    id: 'bank',
    title: 'Rachunki bankowe',
    blocks: [
      { p: 'Dla kont oszczędnościowych i osobistych nie ma wyciągu do importu, więc od czasu do czasu wpisujesz saldo. Te pieniądze liczą się jako gotówka: zasilają poduszkę finansową i widać je w majątku netto i prognozach.' },
      { steps: ['Na ekranie „Przegląd” wybierz „Dodaj rachunek bankowy” i nadaj mu nazwę, np. „Oszczędności PKO”, z walutą i dzisiejszym saldem.', 'Gdy saldo się zmieni, wybierz „Zaktualizuj saldo” i wpisz całe nowe saldo. W księgach zapisuje się tylko różnica, jako wpłata lub wypłata.', 'Dla rachunku w innej walucie podaj kurs wymiany z tego dnia.'] },
    ],
  },
  {
    id: 'metals',
    title: 'Fizyczne złoto i srebro',
    blocks: [
      { p: 'Monety i sztabki, które trzymasz sam, są liczone w gramach czystego metalu, więc ich koszt, wartość i zyski działają jak w przypadku akcji. Fundusz złota notowany na giełdzie, np. Xetra-Gold, to zwykła pozycja: kup go na ekranie „Transakcja”.' },
      { steps: ['Na ekranie „Przegląd” wybierz „Dodaj monety lub sztabki”. Wpisz ilość czystego metalu w uncjach troy lub gramach (moneta jednouncjowa zawiera jedną uncję, nawet jeśli waży więcej) i wszystko, co zapłaciłeś, łącznie z marżą dealera.', 'Od czasu do czasu zaktualizuj cenę, za uncję lub za gram. Do tego czasu używana jest cena, którą zapłaciłeś.', 'Przy sprzedaży wpisz kwotę otrzymaną po potrąceniu marży dealera. Koszt zdejmowany jest najpierw z najstarszych zakupów.'] },
      { p: 'Na ekranie „Podatki i rozliczenia” przy każdej sprzedaży widać, kiedy metal został kupiony, bo w niektórych krajach opodatkowanie złota zależy od okresu posiadania. W Bułgarii robocze przepisy traktują je jako wolne od podatku; potwierdź to u księgowego.' },
    ],
  },
  {
    id: 'private',
    title: 'Inwestycje prywatne i ich wypłata',
    blocks: [
      { p: 'Pieniądze w czymś bez ceny rynkowej, np. współinwestycja z pracodawcą albo udział w prywatnej spółce. Liczą się w majątku netto według tego, ile zapłaciłeś, albo według wartości, którą ustawisz, i nie wchodzą do prognoz, bo nie możesz ich sprzedać, kiedy chcesz.' },
      { steps: ['Na ekranie „Przegląd” wybierz „Dodaj inwestycję prywatną”: ile zapłaciłeś, kiedy, w jakiej walucie i po jakim kursie oraz jak będą opodatkowane pieniądze, które wrócą.', 'Ustaw wartość, gdy dowiesz się, ile jest warta.', 'Gdy pieniądze wrócą, wybierz „Zapisz wypłatę”: kwota przed opodatkowaniem, podatek pobrany przez pracodawcę oraz czy została opodatkowana jako dochód ze stosunku pracy, czy jako zysk kapitałowy.'] },
      { p: 'Inwestycja się wtedy zamyka. To, co wpłaciłeś, wraca do ciebie, a różnica jest księgowana jako dochód lub zysk. Dochód ze stosunku pracy ma własną sekcję na ekranie „Podatki i rozliczenia”; zysk kapitałowy jest wymieniony razem z pozostałymi sprzedażami. Jeśli pieniądze trafiły na rachunek bankowy, który tu śledzisz, zaktualizuj też jego saldo. „Cofnij wypłatę” usuwa ją z ksiąg.' },
    ],
  },
  {
    id: 'claude',
    title: 'Zapytaj asystenta AI',
    blocks: [
      { p: 'Asystent AI może czytać twój portfel, odpowiadać na pytania liczbami z aplikacji, pomagać w planowaniu i zapisywać twoje decyzje, gdy je zatwierdzisz. Jest opcjonalny: wszystko w Investor działa bez niego.' },
      { p: 'W aplikacji desktopowej kliknij przycisk ✦ w prawym dolnym rogu i wybierz model: na swoim komputerze przez Ollama lub LM Studio (nic nie opuszcza komputera) albo Anthropic lub OpenAI z własnym kluczem API. Każda zmiana, którą chce wprowadzić, pokazuje kartę do zatwierdzenia i staje się niezapisaną zmianą, którą zapisujesz jak zwykle. Na stronie internetowej przycisk wyjaśnia, dlaczego potrzebna jest aplikacja desktopowa: twój portfel nie powinien przechodzić przez stronę internetową w drodze do modelu.' },
      { p: 'Możesz też użyć własnego narzędzia AI, np. Claude Code, Cursor lub VS Code. W aplikacji desktopowej otwórz asystenta, kliknij u góry przycisk z wtyczką i wybierz „Włącz”. Skopiuj pokazane polecenie do Claude Code albo adres do ustawień MCP innego narzędzia. To wszystko: bez instalacji i bez zapisywania hasła.' },
      { list: ['Działa, dopóki aplikacja jest otwarta, a plik odblokowany. Gdy zablokujesz plik lub zamkniesz aplikację, twoje narzędzie nie może go już czytać.', 'Każda zmiana, którą twoje narzędzie chce wprowadzić, pokazuje tu kartę do zatwierdzenia i staje się niezapisaną zmianą, którą zapisujesz jak zwykle.', 'Adres zawiera sekret. Jeśli udostępniłeś go przez pomyłkę, utwórz nowy adres. Przy modelu w chmurze to, co czyta twoje narzędzie, trafia do tego dostawcy.', 'W Claude Code polecenie /plugin install investor --marketplace dobrinov/investor dodaje gotowe umiejętności: przegląd miesięczny, przećwiczenie transakcji, sezon podatkowy i emeryturę.'] },
      { p: 'O co możesz zapytać:' },
      { list: ['„Co mam w portfelu i ile jest w gotówce?”', '„Porównaj zainwestowanie 10 000 € naraz z rozłożeniem na 12 miesięcy, w horyzoncie 20 lat.”', '„Sprzedałem dziś 2 VUAA na Xetra po 131,20. Zapisz to i powiedz, ile wyniesie podatek.”', '„Czy jestem na dobrej drodze do moich celów? Co trzeba zrobić, żeby przejść na emeryturę w wieku 60 lat?”'] },
    ],
  },
  {
    id: 'export-ibkr',
    title: 'Eksport z Interactive Brokers',
    blocks: [
      { p: 'Investor odczytuje Activity Flex Query w formacie CSV. Konfigurujesz je raz i uruchamiasz, gdy chcesz mieć świeże dane.' },
      {
        steps: [
          'W IBKR Portal otwórz Performance & Reports → Flex Queries.',
          'Obok Activity Flex Query kliknij + i nazwij zapytanie „investor-export”.',
          'Zaznacz te sekcje i w każdej kliknij Select All: Account Information, Trades (Executions), Cash Transactions, Transaction Fees, Open Positions (Summary), Corporate Actions, Transfers, Financial Instrument Information, Conversion Rates.',
          'Delivery configuration: Format CSV; Include header and trailer records — No; Include column headers — Yes; Include section code and line descriptor — Yes; Date format yyyy-MM-dd; Time format HH:mm:ss; Date/time separator „;”.',
          'Zapisz i kliknij Run. Wybierz Custom Date Range od pierwszej wpłaty do dziś. Jeśli IBKR ogranicza zakres do 365 dni, uruchom zapytanie osobno dla każdego roku.',
          'Zaimportuj tu pobrany plik. Ponowne uruchomienie później jest bezpieczne: wiersze już zaimportowane są pomijane.',
        ],
      },
      { p: 'Jeśli eksportujesz tylko otwarte pozycje, trafiają one do ksiąg jako bilans otwarcia, bez dat zakupu i opłat. Późniejszy import pełnej historii automatycznie je zastępuje.' },
    ],
  },
  {
    id: 'export-revolut',
    title: 'Eksport z Revolut',
    blocks: [
      {
        steps: [
          'W aplikacji Revolut otwórz Invest (lub Stocks).',
          'Otwórz ⋯ lub Documents → Brokerage account → Account statement.',
          'Wybierz Excel/CSV i pełny zakres dat, a potem zaimportuj tu plik.',
        ],
      },
      { p: 'Wyciągi z Revolut nie podają, na której giełdzie odbyła się transakcja. W Bułgarii od tego zależy, czy zysk jest wolny od podatku, więc takie sprzedaże są oznaczone na ekranie „Podatki i rozliczenia” jako „Sprawdź miejsce”.' },
    ],
  },
  {
    id: 'add-broker',
    title: 'Brakuje twojego brokera lub giełdy?',
    blocks: [
      { p: 'Każdego brokera opisuje mały plik konfiguracyjny, więc dodanie nowego nie wymaga programowania. Są dwa sposoby.' },
      { p: 'Poproś o niego. Otwórz zgłoszenie (issue) na GitHubie z nazwą brokera i zanonimizowanym eksportem:' },
      {
        steps: [
          'Wyeksportuj wyciąg, w którym jest co najmniej wpłata, zakup, sprzedaż, dywidenda i opłata, jeśli je masz.',
          'Zastąp numer rachunku, imię i nazwisko oraz kwoty zmyślonymi wartościami. Zachowaj nazwy kolumn, format dat i etykiety typów dokładnie tak, jak zostały wyeksportowane — to je odczytuje konfiguracja.',
          'Otwórz nowe zgłoszenie na GitHubie i dołącz plik.',
        ],
      },
      { link: 'Otwórz zgłoszenie na GitHubie', href: 'https://github.com/dobrinov/investor/issues/new' },
      { p: 'Dodaj go sam. Napisz plik konfiguracyjny taki jak ten, umieść go w folderze brokers/ projektu i otwórz pull request. Wiersze, których konfiguracja nie rozpoznaje, są podczas importu wypisywane z numerami, więc widzisz, czego jeszcze brakuje.' },
      { code: brokerExample },
      { p: 'Rodzaje wierszy: buy, sell, deposit, withdrawal, cash_transfer, dividend, withholding_tax, fee, interest, fx_pair_trade, position_snapshot, ignore. Rodzaje opłat: commission, fx_conversion, exchange, transaction_tax, custody, interest, other.' },
    ],
  },
  {
    id: 'add-tax',
    title: 'Brakuje twojego urzędu skarbowego?',
    blocks: [
      { p: 'Przepisy podatkowe są w pakietach: jeden plik na kraj i rok podatkowy, w folderze jurisdictions/ projektu (np. jurisdictions/bg/2026.toml). Pakiet określa stawki, zwolnienia, formularze i terminy.' },
      {
        steps: [
          'Otwórz zgłoszenie na GitHubie z nazwą kraju i rokiem podatkowym i podlinkuj oficjalne wytyczne organu podatkowego.',
          'Albo skopiuj pakiet bułgarski, zmień wartości i otwórz pull request.',
          'Zostaw verified = false, dopóki doradca podatkowy nie sprawdzi pakietu. Aplikacja pokazuje ostrzeżenie przy niesprawdzonych pakietach.',
        ],
      },
      { code: packExample },
      { p: 'Powyższe wartości są tylko przykładowe. Każda stawka, zwolnienie i nazwa formularza musi pochodzić z oficjalnych źródeł.' },
      { link: 'Otwórz zgłoszenie na GitHubie', href: 'https://github.com/dobrinov/investor/issues/new' },
    ],
  },
  {
    id: 'glossary',
    title: 'Słowniczek',
    blocks: [
      {
        terms: [
          ['Księgowość podwójna', 'Każdy ruch pieniędzy jest zapisywany dwa razy — skąd przyszły i dokąd poszły — więc księgi zawsze się bilansują.'],
          ['Zapis księgowy', 'Jedno zdarzenie, np. zakup lub dywidenda, zapisane jako księgowania, które sumują się do zera.'],
          ['Zestawienie obrotów i sald', 'Saldo każdego konta. Gdy księgi są poprawne, strona Winien równa się stronie Ma.'],
          ['Koszt nabycia', 'Ile zapłaciłeś za inwestycję, łącznie z opłatami, jeśli przepisy na to pozwalają.'],
          ['FIFO', '„Pierwsze weszło, pierwsze wyszło”: przy sprzedaży za sprzedane uznaje się najpierw najstarsze kupione jednostki.'],
          ['Zysk zrealizowany', 'Zysk lub strata utrwalone przez sprzedaż. To właśnie podlega opodatkowaniu.'],
          ['Zysk niezrealizowany', 'Zysk lub strata na inwestycjach, które nadal masz, według ich ostatniej ceny.'],
          ['TER', 'Całkowity wskaźnik kosztów: roczny koszt utrzymania funduszu, potrącany z jego ceny.'],
          ['Podatek u źródła', 'Podatek pobrany przez inny kraj, zanim dywidenda do ciebie trafi. Często można go częściowo odliczyć w kraju.'],
          ['Miejsce obrotu', 'Giełda, na której odbyła się transakcja, np. Xetra lub NASDAQ.'],
          ['ETF', 'Fundusz notowany na giełdzie jak akcja, zwykle obejmujący wiele spółek naraz.'],
          ['Poduszka finansowa', 'Pieniądze trzymane w gotówce na niespodzianki, żeby nigdy nie trzeba było sprzedawać inwestycji w złym momencie.'],
        ],
      },
    ],
  },
]
