import { brokerExample, packExample } from './en'
import type { Section } from './types'

export const pt: Section[] = [
  {
    id: 'start',
    title: 'Primeiros passos',
    blocks: [
      { p: 'O Investor guarda a sua contabilidade, os seus planos e os seus valores fiscais num único ficheiro encriptado no seu computador. Uma primeira sessão sensata:' },
      {
        steps: [
          'Exporte os extratos de cada corretora (veja as secções seguintes).',
          'Importe-os no ecrã «Importar extratos». Cada transação, dividendo, comissão e imposto torna-se um lançamento equilibrado.',
          'Veja o «Resumo». As posições sem preço contam pelo que pagou; adicione um preço com «Atualizar preço».',
          'Preencha o seu perfil e adicione objetivos no ecrã «Objetivos».',
          'Explore as «Projeções» e, antes de entregar a declaração, «Impostos e relatórios».',
          'Guarde o ficheiro .invest e mantenha uma cópia de segurança.',
        ],
      },
    ],
  },
  {
    id: 'file',
    title: 'O seu ficheiro, a sua palavra-passe',
    blocks: [
      { p: 'Não há conta nem servidor. Tudo vive no seu ficheiro .invest, encriptado com uma chave criada a partir da sua palavra-passe. Sem a palavra-passe, o ficheiro é ilegível — para qualquer pessoa, incluindo nós.' },
      {
        list: [
          'Guarde o ficheiro na pasta Documentos e uma cópia numa pen USB ou numa pasta na nuvem que já utilize. A cópia continua encriptada.',
          'Não é possível repor a palavra-passe. Anote-a num local seguro, ou use um gestor de palavras-passe.',
          'As alterações por guardar ficam encriptadas neste navegador, para que recarregar a página não as perca. Limpar os dados do navegador apaga essa cópia — guarde o ficheiro para manter o seu trabalho.',
          'Alterar a palavra-passe («Definições») só afeta as cópias guardadas depois. As cópias antigas continuam a abrir com a palavra-passe antiga, por isso substitua-as.',
        ],
      },
    ],
  },
  {
    id: 'example',
    title: 'Experimente primeiro a carteira de exemplo',
    blocks: [
      { p: 'O exemplo é um investidor inventado, com fundos de índice, ações, um fundo de obrigações, ouro em bolsa e em moedas, criptomoedas, uma conta poupança, uma participação numa startup e objetivos. Todos os ecrãs funcionam, por isso pode ver o que a aplicação faz antes de importar seja o que for.' },
      { list: ['Abra-o no ecrã inicial com «Experimentar a carteira de exemplo», ou vá ao endereço que termina em #example.', 'Altere o que quiser: nada é guardado, e a cópia do seu próprio ficheiro neste navegador nunca é tocada.', '«Fechar exemplo» leva-o de volta ao ecrã inicial.'] },
    ],
  },
  {
    id: 'bank',
    title: 'Contas bancárias',
    blocks: [
      { p: 'As contas poupança e as contas à ordem não têm extrato para importar, por isso escreve o saldo de vez em quando. O dinheiro conta como liquidez: preenche o fundo de emergência e aparece no património líquido e nas projeções.' },
      { steps: ['No «Resumo», escolha «Adicionar conta bancária» e dê-lhe um nome, como «UBB poupança», com a moeda e o saldo de hoje.', 'Sempre que o saldo mudar, escolha «Atualizar saldo» e escreva o novo saldo completo. A contabilidade regista apenas a diferença, como dinheiro que entrou ou saiu.', 'Para uma conta noutra moeda, indique a taxa de câmbio desse dia.'] },
    ],
  },
  {
    id: 'metals',
    title: 'Ouro e prata físicos',
    blocks: [
      { p: 'As moedas e barras que guarda consigo contam em gramas de metal puro, por isso o custo, o valor e as mais-valias funcionam como nas ações. Um fundo de ouro negociado em bolsa, como o Xetra-Gold, é uma posição normal: compre-o no ecrã «Negociar».' },
      { steps: ['No «Resumo», escolha «Adicionar moedas ou barras». Indique o metal puro em onças troy ou gramas (uma moeda de uma onça contém uma onça, mesmo que pese mais) e tudo o que pagou, incluindo o prémio do comerciante.', 'Atualize o preço de vez em quando, por onça ou por grama. Até lá, é usado o preço que pagou.', 'Quando vender, registe o que recebeu depois da margem do comerciante. O custo é abatido primeiro às compras mais antigas.'] },
      { p: 'No ecrã de impostos, cada venda mostra quando o metal foi comprado, porque vários países tributam o ouro conforme o tempo de detenção. Na Bulgária, as regras em rascunho tratam-no como isento; confirme isto com um contabilista.' },
    ],
  },
  {
    id: 'private',
    title: 'Participações privadas e o seu pagamento',
    blocks: [
      { p: 'Dinheiro aplicado em algo sem preço de mercado, como um coinvestimento com a entidade patronal ou uma participação numa empresa não cotada. Conta no património líquido pelo que pagou, ou por um valor que definir, e fica fora das projeções porque não o pode vender quando quiser.' },
      { steps: ['No «Resumo», escolha «Adicionar participação privada»: quanto pagou, quando, em que moeda e a que taxa de câmbio, e como será tributado o dinheiro que voltar.', 'Defina um valor sempre que souber quanto vale.', 'Quando o dinheiro voltar, escolha «Registar pagamento»: o montante antes de impostos, o imposto que a entidade patronal reteve e se foi tributado como rendimento do trabalho ou como mais-valia.'] },
      { p: 'A participação fecha. O que aplicou volta para si, e a diferença é registada como rendimento ou como mais-valia. Os rendimentos do trabalho têm a sua própria secção no ecrã de impostos; uma mais-valia aparece junto das suas outras vendas. Se o dinheiro foi para uma conta bancária que acompanha aqui, atualize também o saldo. «Anular pagamento» retira-o de novo.' },
    ],
  },
  {
    id: 'claude',
    title: 'Pergunte a um assistente de IA',
    blocks: [
      { p: 'Um assistente de IA pode ler a sua carteira, responder a perguntas com os números da própria aplicação, ajudá-lo a planear e registar o que decidir, depois de o aprovar. É opcional: tudo no Investor funciona sem ele.' },
      { p: 'Na aplicação de computador, carregue no botão ✦ no canto inferior direito e escolha um modelo: um no seu computador, através do Ollama ou do LM Studio (nada sai do seu computador), ou a Anthropic ou a OpenAI com a sua própria chave de API. Cada alteração que queira fazer mostra um cartão para aprovar e passa a ser uma alteração por guardar, que guarda como habitualmente. No site, o botão explica porque é precisa a aplicação de computador: a sua carteira não deve passar por uma página web a caminho de um modelo.' },
      { p: 'Também pode usar a sua própria ferramenta de IA, como o Claude Code, o Cursor ou o VS Code. Na aplicação de computador, abra o assistente, carregue no botão da ficha, no topo, e escolha «Ativar». Copie o comando apresentado para o Claude Code, ou o endereço para as definições de MCP de outra ferramenta. É só isso: nada para instalar, nenhuma palavra-passe para guardar.' },
      { list: ['Funciona enquanto a aplicação estiver aberta e o ficheiro desbloqueado. Se bloquear o ficheiro ou fechar a aplicação, a sua ferramenta deixa de o conseguir ler.', 'Cada alteração que a sua ferramenta queira fazer mostra aqui um cartão para aprovar e passa a ser uma alteração por guardar, que guarda como habitualmente.', 'O endereço contém um segredo. Crie um endereço novo se o partilhou por engano. Com um modelo na nuvem, o que a sua ferramenta lê é enviado a esse fornecedor.', 'No Claude Code, /plugin install investor --marketplace dobrinov/investor acrescenta competências prontas a usar: uma revisão mensal, ensaiar uma transação, época de impostos e reforma.'] },
      { p: 'Exemplos de perguntas:' },
      { list: ['«O que tenho, e quanto está em liquidez?»', '«Compara investir 10 000 € de uma vez com repartir ao longo de 12 meses, a 20 anos.»', '«Vendi hoje 2 VUAA na Xetra a 131,20. Regista e diz-me o imposto.»', '«Estou no caminho certo para os meus objetivos? O que seria preciso para me reformar aos 60?»'] },
    ],
  },
  {
    id: 'export-ibkr',
    title: 'Exportar da Interactive Brokers',
    blocks: [
      { p: 'O Investor lê uma Activity Flex Query em CSV. Configura-a uma vez e executa-a sempre que quiser dados atualizados.' },
      {
        steps: [
          'No IBKR Portal, abra Performance & Reports → Flex Queries.',
          'Junto a Activity Flex Query, carregue em + e dê-lhe o nome «investor-export».',
          'Marque estas secções e carregue em Select All em cada uma: Account Information, Trades (Executions), Cash Transactions, Transaction Fees, Open Positions (Summary), Corporate Actions, Transfers, Financial Instrument Information, Conversion Rates.',
          'Delivery configuration: Format CSV; Include header and trailer records — No; Include column headers — Yes; Include section code and line descriptor — Yes; Date format yyyy-MM-dd; Time format HH:mm:ss; Date/time separator «;».',
          'Guarde e carregue em Run. Escolha Custom Date Range desde o primeiro depósito até hoje. Se a IBKR limitar o período a 365 dias, execute-a uma vez por ano.',
          'Importe aqui o ficheiro descarregado. Voltar a executá-la mais tarde é seguro: as linhas já importadas são ignoradas.',
        ],
      },
      { p: 'Se exportar apenas as posições abertas, estas entram como saldos iniciais, sem datas de compra nem comissões. Importar mais tarde o histórico completo substitui-as automaticamente.' },
    ],
  },
  {
    id: 'export-revolut',
    title: 'Exportar da Revolut',
    blocks: [
      {
        steps: [
          'Na aplicação da Revolut, abra Invest (ou Stocks).',
          'Abra ⋯ ou Documents → Brokerage account → Account statement.',
          'Escolha Excel/CSV e o período completo, e depois importe aqui o ficheiro.',
        ],
      },
      { p: 'Os extratos da Revolut não indicam em que bolsa foi feita a transação. Na Bulgária, isso decide se a mais-valia está isenta, por isso essas vendas ficam marcadas com «Verificar local» no ecrã de impostos.' },
    ],
  },
  {
    id: 'add-broker',
    title: 'A sua corretora ou bolsa não está aqui?',
    blocks: [
      { p: 'Cada corretora é descrita por um pequeno ficheiro de configuração, por isso adicioná-la não exige programação. Há duas formas.' },
      { p: 'Peça-a. Abra uma issue no GitHub com o nome da corretora e uma exportação anonimizada:' },
      {
        steps: [
          'Exporte um extrato que tenha, se possível, pelo menos um depósito, uma compra, uma venda, um dividendo e uma comissão.',
          'Substitua o número de conta, o nome e os montantes por valores inventados. Mantenha os nomes das colunas, o formato das datas e os tipos de linha exatamente como foram exportados — é isso que a configuração lê.',
          'Abra uma nova issue no GitHub e anexe o ficheiro.',
        ],
      },
      { link: 'Abrir uma issue no GitHub', href: 'https://github.com/dobrinov/investor/issues/new' },
      { p: 'Adicione-a você mesmo. Escreva um ficheiro de configuração como este, coloque-o na pasta brokers/ do projeto e abra um pull request. Durante a importação, as linhas que a configuração não reconhece são listadas pelo número de linha, para que veja o que ainda falta.' },
      { code: brokerExample },
      { p: 'Tipos de linha: buy, sell, deposit, withdrawal, cash_transfer, dividend, withholding_tax, fee, interest, fx_pair_trade, position_snapshot, ignore. Tipos de comissão: commission, fx_conversion, exchange, transaction_tax, custody, interest, other.' },
    ],
  },
  {
    id: 'add-tax',
    title: 'A sua autoridade tributária não está aqui?',
    blocks: [
      { p: 'As regras fiscais vêm em pacotes: um ficheiro por país e ano fiscal, na pasta jurisdictions/ do projeto (por exemplo, jurisdictions/bg/2026.toml). Um pacote define as taxas, as isenções, os formulários e os prazos.' },
      {
        steps: [
          'Abra uma issue no GitHub indicando o país e o ano fiscal, com uma ligação às orientações oficiais da autoridade tributária.',
          'Ou copie o pacote búlgaro, altere os valores e abra um pull request.',
          'Deixe verified = false até um profissional da área fiscal verificar o pacote. A aplicação mostra um aviso para pacotes não verificados.',
        ],
      },
      { code: packExample },
      { p: 'Os valores acima são apenas exemplos. Cada taxa, isenção e nome de formulário tem de vir das fontes oficiais.' },
      { link: 'Abrir uma issue no GitHub', href: 'https://github.com/dobrinov/investor/issues/new' },
    ],
  },
  {
    id: 'glossary',
    title: 'Termos usados na aplicação',
    blocks: [
      {
        terms: [
          ['Contabilidade em partidas dobradas', 'Cada movimento de dinheiro é registado duas vezes — de onde veio e para onde foi — por isso a contabilidade fica sempre equilibrada.'],
          ['Lançamento', 'Um acontecimento, como uma compra ou um dividendo, registado como movimentos que somam zero.'],
          ['Balancete', 'O saldo de cada conta. Quando a contabilidade está correta, os débitos são iguais aos créditos.'],
          ['Custo de aquisição', 'O que pagou por um investimento, incluindo as comissões, quando as regras o permitem.'],
          ['FIFO', 'First in, first out («primeiro a entrar, primeiro a sair»): quando vende, as unidades compradas há mais tempo são as primeiras a ser consideradas vendidas.'],
          ['Mais-valia realizada', 'Lucro ou prejuízo fixado com a venda. É isto que é tributado.'],
          ['Mais-valia não realizada', 'Lucro ou prejuízo em investimentos que ainda detém, ao último preço.'],
          ['TER', 'Total expense ratio (encargos correntes): o custo anual de um fundo, descontado no seu preço.'],
          ['Retenção na fonte', 'Imposto cobrado por outro país antes de o dividendo lhe chegar. Muitas vezes é parcialmente dedutível no seu país.'],
          ['Local de negociação', 'A bolsa onde a transação aconteceu, como a Xetra ou o NASDAQ.'],
          ['ETF', 'Um fundo negociado em bolsa como uma ação, que normalmente detém muitas empresas de uma só vez.'],
          ['Fundo de emergência', 'Dinheiro guardado em liquidez para imprevistos, para nunca ter de vender investimentos num mau momento.'],
        ],
      },
    ],
  },
]
