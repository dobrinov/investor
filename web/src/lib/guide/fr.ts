import { brokerExample, packExample } from './en'
import type { Section } from './types'

export const fr: Section[] = [
  {
    id: 'start',
    title: 'Premiers pas',
    blocks: [
      { p: 'Investor garde vos comptes, vos plans et vos chiffres fiscaux dans un seul fichier chiffré sur votre ordinateur. Pour bien commencer :' },
      {
        steps: [
          'Exportez vos relevés depuis chaque courtier (voir les sections suivantes).',
          'Importez-les dans « Importer des relevés ». Chaque opération, dividende, frais et impôt devient une écriture équilibrée.',
          'Consultez l’« Aperçu ». Les positions sans cours sont comptées à leur prix de revient ; ajoutez un cours avec « Mettre à jour le cours ».',
          'Complétez votre profil et ajoutez des objectifs dans « Objectifs ».',
          'Explorez les « Projections », puis « Impôts et déclarations » avant de remplir votre déclaration.',
          'Enregistrez votre fichier .invest et gardez-en une copie de sauvegarde.',
        ],
      },
    ],
  },
  {
    id: 'file',
    title: 'Votre fichier, votre mot de passe',
    blocks: [
      { p: 'Il n’y a ni compte ni serveur. Tout se trouve dans votre fichier .invest, chiffré avec une clé tirée de votre mot de passe. Sans le mot de passe, le fichier est illisible – pour tout le monde, nous compris.' },
      {
        list: [
          'Gardez le fichier dans votre dossier Documents, et une copie sur une clé USB ou dans un dossier cloud que vous utilisez déjà. La copie reste chiffrée.',
          'Le mot de passe ne peut pas être réinitialisé. Notez-le en lieu sûr, ou utilisez un gestionnaire de mots de passe.',
          'Les modifications non enregistrées sont conservées chiffrées dans ce navigateur : un rechargement ne les fait donc pas perdre. Effacer les données du navigateur supprime cette copie – enregistrez le fichier pour garder votre travail.',
          'Changer le mot de passe (Réglages) ne concerne que les copies enregistrées ensuite. Les anciennes copies s’ouvrent toujours avec l’ancien mot de passe : remplacez-les.',
        ],
      },
    ],
  },
  {
    id: 'example',
    title: 'Essayez d’abord le portefeuille d’exemple',
    blocks: [
      { p: 'L’exemple est un investisseur fictif avec des fonds indiciels, des actions, un fonds obligataire, de l’or en bourse et en pièces, des cryptos, un compte d’épargne, une participation dans une start-up et des objectifs. Chaque écran fonctionne : vous voyez ce que fait l’app avant d’importer quoi que ce soit.' },
      { list: ['Ouvrez-le depuis l’écran d’accueil avec « Essayer le portefeuille d’exemple », ou via l’adresse se terminant par #example.', 'Modifiez ce que vous voulez : rien n’est enregistré, et la copie de votre propre fichier conservée dans ce navigateur n’est jamais touchée.', '« Fermer l’exemple » vous ramène à l’écran d’accueil.'] },
    ],
  },
  {
    id: 'bank',
    title: 'Comptes bancaires',
    blocks: [
      { p: 'Les comptes d’épargne et les comptes courants n’ont pas de relevé à importer : vous saisissez donc le solde de temps en temps. Cet argent compte comme liquidités : il alimente votre épargne de précaution et apparaît dans votre patrimoine net et vos projections.' },
      { steps: ['Dans l’« Aperçu », choisissez « Ajouter un compte bancaire » et donnez-lui un nom, par exemple « Livret UBB », avec sa devise et le solde du jour.', 'Quand le solde change, choisissez « Mettre à jour le solde » et saisissez le nouveau solde complet. Les comptes n’enregistrent que la différence, comme un apport ou un retrait.', 'Pour un compte dans une autre devise, indiquez le taux de change du jour.'] },
    ],
  },
  {
    id: 'metals',
    title: 'Or et argent physiques',
    blocks: [
      { p: 'Les pièces et lingots que vous gardez vous-même sont comptés en grammes de métal fin : prix de revient, valeur et plus-values fonctionnent comme pour des actions. Un produit sur l’or coté en bourse, comme Xetra-Gold, est en revanche une position classique : achetez-le dans « Préparer un ordre ».' },
      { steps: ['Dans l’« Aperçu », choisissez « Ajouter des pièces ou lingots ». Saisissez le métal fin en onces troy ou en grammes (une pièce d’une once contient une once, même si elle pèse davantage) et tout ce que vous avez payé, prime du négociant comprise.', 'Mettez le prix à jour de temps en temps, par once ou par gramme. En attendant, c’est le prix payé qui est utilisé.', 'Quand vous vendez, saisissez ce que vous avez reçu après la marge du négociant. Le prix de revient est imputé d’abord sur vos achats les plus anciens.'] },
      { p: 'Dans l’écran Impôts, chaque vente indique quand le métal a été acheté, car plusieurs pays imposent l’or selon la durée de détention. En Bulgarie, les règles provisoires le considèrent comme exonéré ; vérifiez-le auprès d’un comptable.' },
    ],
  },
  {
    id: 'private',
    title: 'Participations privées et leur versement',
    blocks: [
      { p: 'De l’argent placé dans quelque chose sans prix de marché, comme un co-investissement avec votre employeur ou une participation dans une société non cotée. Il compte dans votre patrimoine net au montant payé, ou à une valeur que vous fixez, et reste hors des projections, car vous ne pouvez pas le vendre quand vous voulez.' },
      { steps: ['Dans l’« Aperçu », choisissez « Ajouter une participation privée » : ce que vous avez payé, quand, dans quelle devise et à quel taux de change, et comment l’argent qui reviendra sera imposé.', 'Fixez une valeur dès que vous savez ce qu’elle vaut.', 'Quand l’argent revient, choisissez « Enregistrer le versement » : le montant avant impôt, l’impôt éventuellement retenu par votre employeur, et s’il a été imposé comme revenu salarial ou comme plus-value.'] },
      { p: 'La position est alors clôturée. Ce que vous aviez versé vous revient, et la différence est comptabilisée comme revenu ou comme plus-value. Les revenus salariaux ont leur propre section dans l’écran Impôts ; une plus-value figure avec vos autres ventes. Si l’argent est arrivé sur un compte bancaire suivi ici, mettez aussi son solde à jour. « Annuler le versement » le retire.' },
    ],
  },
  {
    id: 'claude',
    title: 'Interroger un assistant IA',
    blocks: [
      { p: 'Un assistant IA peut lire votre portefeuille, répondre à vos questions avec les chiffres de l’app, vous aider à planifier et enregistrer vos décisions une fois que vous les avez approuvées. Il est facultatif : tout dans Investor fonctionne sans lui.' },
      { p: 'Dans l’app de bureau, appuyez sur le bouton ✦ en bas à droite et choisissez un modèle : un modèle sur votre ordinateur via Ollama ou LM Studio (rien ne quitte votre ordinateur), ou Anthropic ou OpenAI avec votre propre clé API. Chaque modification qu’il veut faire s’affiche sous forme de carte à approuver, puis devient une modification non enregistrée que vous enregistrez comme d’habitude. Sur le site, le bouton explique pourquoi l’app de bureau est nécessaire : votre portefeuille ne devrait pas passer par une page web pour atteindre un modèle.' },
      { p: 'Vous pouvez aussi utiliser votre propre outil d’IA, comme Claude Code, Cursor ou VS Code. Dans l’app de bureau, ouvrez l’assistant, appuyez sur le bouton en forme de prise en haut et choisissez « Activer ». Copiez la commande affichée dans Claude Code, ou l’adresse dans les réglages MCP d’un autre outil. C’est tout : rien à installer, aucun mot de passe à stocker.' },
      { list: ['Cela fonctionne tant que l’app est ouverte et votre fichier déverrouillé. Verrouillez le fichier ou fermez l’app, et votre outil ne peut plus le lire.', 'Chaque modification que votre outil veut faire s’affiche ici sous forme de carte à approuver, puis devient une modification non enregistrée que vous enregistrez comme d’habitude.', 'L’adresse contient un secret. Créez-en une nouvelle si vous l’avez partagée par erreur. Avec un modèle dans le cloud, ce que lit votre outil est envoyé à ce fournisseur.', 'Dans Claude Code, /plugin install investor --marketplace dobrinov/investor ajoute des compétences prêtes à l’emploi : bilan mensuel, test d’un ordre, saison des impôts et retraite.'] },
      { p: 'Exemples de questions :' },
      { list: ['« Qu’est-ce que je détiens, et combien en liquidités ? »', '« Compare investir 10 000 € en une fois et l’étaler sur 12 mois, sur 20 ans. »', '« J’ai vendu 2 VUAA sur Xetra à 131,20 aujourd’hui. Enregistre-le et dis-moi l’impôt. »', '« Suis-je en bonne voie pour mes objectifs ? Que faudrait-il pour partir à la retraite à 60 ans ? »'] },
    ],
  },
  {
    id: 'export-ibkr',
    title: 'Exporter depuis Interactive Brokers',
    blocks: [
      { p: 'Investor lit une Activity Flex Query au format CSV. Vous la configurez une fois et la lancez chaque fois que vous voulez des données à jour.' },
      {
        steps: [
          'Dans l’IBKR Portal, ouvrez Performance & Reports → Flex Queries.',
          'À côté d’Activity Flex Query, appuyez sur + et nommez-la « investor-export ».',
          'Cochez ces sections et appuyez sur Select All dans chacune : Account Information, Trades (Executions), Cash Transactions, Transaction Fees, Open Positions (Summary), Corporate Actions, Transfers, Financial Instrument Information, Conversion Rates.',
          'Delivery configuration : Format CSV ; Include header and trailer records – No ; Include column headers – Yes ; Include section code and line descriptor – Yes ; Date format yyyy-MM-dd ; Time format HH:mm:ss ; Date/time separator « ; ».',
          'Enregistrez, puis appuyez sur Run. Choisissez Custom Date Range, de votre premier versement à aujourd’hui. Si IBKR limite la période à 365 jours, lancez-la une fois par année.',
          'Importez ici le fichier téléchargé. La relancer plus tard ne pose aucun problème : les lignes déjà importées sont ignorées.',
        ],
      },
      { p: 'Si vous n’exportez que les positions ouvertes, elles arrivent comme soldes d’ouverture, sans dates d’achat ni frais. Importer plus tard l’historique complet les remplace automatiquement.' },
    ],
  },
  {
    id: 'export-revolut',
    title: 'Exporter depuis Revolut',
    blocks: [
      {
        steps: [
          'Dans l’app Revolut, ouvrez Invest (ou Stocks).',
          'Ouvrez ⋯ ou Documents → Brokerage account → Account statement.',
          'Choisissez Excel/CSV et toute la période, puis importez le fichier ici.',
        ],
      },
      { p: 'Les relevés Revolut n’indiquent pas sur quelle bourse un ordre a été exécuté. En Bulgarie, c’est ce qui décide si une plus-value est exonérée : ces ventes sont donc signalées « Vérifier la place » dans l’écran Impôts.' },
    ],
  },
  {
    id: 'add-broker',
    title: 'Votre courtier ou votre bourse manque ?',
    blocks: [
      { p: 'Chaque courtier est décrit par un petit fichier de configuration : en ajouter un ne demande aucune programmation. Il y a deux façons de faire.' },
      { p: 'Demandez-le. Ouvrez une issue sur GitHub avec le nom du courtier et un export anonymisé :' },
      {
        steps: [
          'Exportez un relevé contenant au moins un versement, un achat, une vente, un dividende et des frais, si vous en avez.',
          'Remplacez votre numéro de compte, votre nom et les montants par des valeurs inventées. Gardez les noms de colonnes, le format de date et les libellés de type exactement tels qu’exportés – c’est ce que lit la configuration.',
          'Ouvrez une nouvelle issue sur GitHub et joignez le fichier.',
        ],
      },
      { link: 'Ouvrir une issue sur GitHub', href: 'https://github.com/dobrinov/investor/issues/new' },
      { p: 'Ajoutez-le vous-même. Écrivez un fichier de configuration comme celui-ci, placez-le dans le dossier brokers/ du projet et ouvrez une pull request. Les lignes que votre configuration ne reconnaît pas sont listées avec leur numéro lors de l’import : vous voyez ainsi ce qui manque encore.' },
      { code: brokerExample },
      { p: 'Types de lignes : buy, sell, deposit, withdrawal, cash_transfer, dividend, withholding_tax, fee, interest, fx_pair_trade, position_snapshot, ignore. Types de frais : commission, fx_conversion, exchange, transaction_tax, custody, interest, other.' },
    ],
  },
  {
    id: 'add-tax',
    title: 'Votre administration fiscale manque ?',
    blocks: [
      { p: 'Les règles fiscales sont regroupées en packs : un fichier par pays et par année fiscale, dans le dossier jurisdictions/ du projet (par exemple jurisdictions/bg/2026.toml). Un pack définit les taux, les exonérations, les formulaires et les échéances.' },
      {
        steps: [
          'Ouvrez une issue sur GitHub en indiquant le pays et l’année fiscale, avec un lien vers la documentation officielle de l’administration fiscale.',
          'Ou copiez le pack bulgare, modifiez les valeurs et ouvrez une pull request.',
          'Laissez verified = false tant qu’un fiscaliste n’a pas vérifié le pack. L’app affiche un avertissement pour les packs non vérifiés.',
        ],
      },
      { code: packExample },
      { p: 'Les valeurs ci-dessus sont des exemples. Chaque taux, exonération et nom de formulaire doit provenir des sources officielles.' },
      { link: 'Ouvrir une issue sur GitHub', href: 'https://github.com/dobrinov/investor/issues/new' },
    ],
  },
  {
    id: 'glossary',
    title: 'Les mots de l’app',
    blocks: [
      {
        terms: [
          ['Comptabilité en partie double', 'Chaque mouvement d’argent est enregistré deux fois – d’où il vient et où il va – pour que les comptes soient toujours équilibrés.'],
          ['Écriture comptable', 'Un événement, comme un achat ou un dividende, enregistré en lignes dont la somme est nulle.'],
          ['Balance', 'Le solde de chaque compte. Quand les comptes sont justes, le total des débits égale celui des crédits.'],
          ['Prix de revient', 'Ce que vous avez payé pour un placement, frais compris lorsque les règles le permettent.'],
          ['FIFO', 'Premier entré, premier sorti : quand vous vendez, les parts achetées en premier sont considérées comme vendues en premier.'],
          ['Plus-value réalisée', 'Gain ou perte constaté par une vente. C’est ce qui est imposé.'],
          ['Plus-value latente', 'Gain ou perte sur les placements que vous détenez encore, à leur dernier cours.'],
          ['TER', 'Frais courants totaux : le coût annuel de fonctionnement d’un fonds, prélevé sur sa valeur.'],
          ['Retenue à la source', 'Impôt prélevé par un autre pays avant qu’un dividende ne vous parvienne. Souvent en partie déductible dans votre pays.'],
          ['Place d’exécution', 'La bourse où un ordre a été exécuté, comme Xetra ou le NASDAQ.'],
          ['ETF', 'Un fonds coté en bourse et négocié comme une action, qui détient généralement de nombreuses entreprises à la fois.'],
          ['Épargne de précaution', 'De l’argent gardé en liquidités pour les imprévus, afin de ne jamais devoir vendre vos placements au mauvais moment.'],
        ],
      },
    ],
  },
]
