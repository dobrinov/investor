import { brokerExample, packExample } from './en'
import type { Section } from './types'

export const es: Section[] = [
  {
    id: 'start',
    title: 'Primeros pasos',
    blocks: [
      { p: 'Investor guarda tu contabilidad, tus planes y tus cifras fiscales en un único archivo cifrado en tu ordenador. Una buena primera sesión:' },
      {
        steps: [
          'Exporta tus extractos de cada bróker (consulta las secciones siguientes).',
          'Impórtalos en la pantalla Importar extractos. Cada operación, dividendo, comisión e impuesto se convierte en un asiento cuadrado.',
          'Revisa el Resumen. Las posiciones sin precio se cuentan por lo que pagaste; añade un precio con «Actualizar precio».',
          'Rellena tu perfil y añade objetivos en la pantalla Objetivos.',
          'Explora Proyecciones, y también Impuestos e informes antes de presentar la declaración.',
          'Guarda tu archivo .invest y ten una copia de seguridad.',
        ],
      },
    ],
  },
  {
    id: 'file',
    title: 'Tu archivo, tu contraseña',
    blocks: [
      { p: 'No hay cuenta ni servidor. Todo vive en tu archivo .invest, cifrado con una clave que se obtiene de tu contraseña. Sin la contraseña el archivo es ilegible para cualquiera, también para nosotros.' },
      {
        list: [
          'Guarda el archivo en tu carpeta Documentos y una copia en un USB o en una carpeta en la nube que ya uses. La copia sigue cifrada.',
          'No se puede restablecer la contraseña. Apúntala en un lugar seguro o usa un gestor de contraseñas.',
          'Los cambios sin guardar se conservan cifrados en este navegador para que una recarga no los pierda. Si borras los datos del navegador, esa copia desaparece: guarda el archivo para conservar tu trabajo.',
          'Cambiar la contraseña (Ajustes) solo afecta a las copias guardadas después. Las copias antiguas se siguen abriendo con la contraseña anterior, así que sustitúyelas.',
        ],
      },
    ],
  },
  {
    id: 'example',
    title: 'Prueba antes la cartera de ejemplo',
    blocks: [
      { p: 'El ejemplo es un inversor inventado con fondos indexados, acciones, un fondo de renta fija, oro en bolsa y en monedas, criptomonedas, una cuenta de ahorro, una participación en una startup y objetivos. Todas las pantallas funcionan, así que puedes ver qué hace la aplicación antes de importar nada.' },
      { list: ['Ábrelo desde la pantalla de inicio con «Prueba la cartera de ejemplo», o ve a la dirección que termina en #example.', 'Cambia lo que quieras: no se guarda nada y la copia de tu propio archivo en este navegador no se toca.', '«Cerrar ejemplo» te devuelve a la pantalla de inicio.'] },
    ],
  },
  {
    id: 'bank',
    title: 'Cuentas bancarias',
    blocks: [
      { p: 'Las cuentas de ahorro y las cuentas corrientes no tienen extracto que importar, así que escribes el saldo de vez en cuando. El dinero cuenta como efectivo: llena tu fondo de emergencia y aparece en tu patrimonio neto y en las proyecciones.' },
      { steps: ['En Resumen, elige «Añadir una cuenta bancaria» y ponle un nombre, como «Cuenta de ahorro», con su moneda y el saldo de hoy.', 'Cada vez que cambie el saldo, elige «Actualizar saldo» y escribe el saldo nuevo completo. La contabilidad solo registra la diferencia, como dinero aportado o retirado.', 'Para una cuenta en otra moneda, indica el tipo de cambio de ese día.'] },
    ],
  },
  {
    id: 'metals',
    title: 'Oro y plata físicos',
    blocks: [
      { p: 'Las monedas y los lingotes que guardas tú se cuentan en gramos de metal puro, así que su coste, su valor y sus ganancias funcionan como los de las acciones. Un fondo de oro que cotiza en bolsa, como Xetra-Gold, es en cambio una posición normal: cómpralo en la pantalla Operar.' },
      { steps: ['En Resumen, elige «Añadir monedas o lingotes». Indica el metal puro en onzas troy o en gramos (una moneda de una onza contiene una onza aunque pese más) y todo lo que pagaste, incluida la prima del vendedor.', 'Actualiza el precio de vez en cuando, por onza o por gramo. Mientras no lo hagas, se usa el precio que pagaste.', 'Cuando vendas, registra lo que recibiste después del margen del comprador. El coste se descuenta primero de tus compras más antiguas.'] },
      { p: 'En la pantalla de Impuestos cada venta muestra cuándo se compró el metal, porque varios países gravan el oro según el tiempo que lo has tenido. En Bulgaria, las normas en borrador lo tratan como exento; compruébalo con un asesor.' },
    ],
  },
  {
    id: 'private',
    title: 'Inversiones privadas y su cobro',
    blocks: [
      { p: 'Dinero en algo sin precio de mercado, como una coinversión con tu empresa o una participación en una sociedad no cotizada. Cuenta en tu patrimonio neto por lo que pagaste, o por el valor que fijes, y queda fuera de las proyecciones porque no puedes venderlo cuando quieras.' },
      { steps: ['En Resumen, elige «Añadir una inversión privada»: cuánto pagaste, cuándo, en qué moneda y a qué tipo de cambio, y cómo tributará el dinero que vuelva.', 'Fija un valor cada vez que sepas cuánto vale.', 'Cuando vuelva el dinero, elige «Registrar cobro»: el importe bruto, la retención que haya aplicado tu empresa y si tributó como rendimiento del trabajo o como ganancia patrimonial.'] },
      { p: 'Entonces la inversión se cierra. Lo que aportaste vuelve a ti y la diferencia se registra como renta o como ganancia. Los rendimientos del trabajo tienen su propia sección en la pantalla de Impuestos; una ganancia patrimonial aparece con tus demás ventas. Si el dinero fue a una cuenta bancaria que sigues aquí, actualiza también su saldo. «Deshacer cobro» lo quita de nuevo.' },
    ],
  },
  {
    id: 'claude',
    title: 'Pregunta a un asistente de IA',
    blocks: [
      { p: 'Un asistente de IA puede leer tu cartera, responder preguntas con las cifras de la propia aplicación, ayudarte a planificar y registrar lo que decidas una vez que lo apruebes. Es opcional: todo en Investor funciona sin él.' },
      { p: 'En la aplicación de escritorio, pulsa el botón ✦ de la esquina inferior derecha y elige un modelo: uno en tu ordenador mediante Ollama o LM Studio (nada sale de tu ordenador), o Anthropic u OpenAI con tu propia clave de API. Cada cambio que quiera hacer muestra una tarjeta para aprobarlo y pasa a ser un cambio sin guardar que guardas como siempre. En la web, el botón explica por qué hace falta la aplicación de escritorio: tu cartera no debería pasar por una página web de camino a un modelo.' },
      { p: 'También puedes usar tu propia herramienta de IA, como Claude Code, Cursor o VS Code. En la aplicación de escritorio, abre el asistente, pulsa el botón del enchufe de arriba y elige Activar. Copia el comando que aparece en Claude Code, o la dirección en los ajustes de MCP de otra herramienta. Eso es todo: sin instalar nada y sin contraseñas que guardar.' },
      { list: ['Funciona mientras la aplicación está abierta y tu archivo desbloqueado. Si bloqueas el archivo o cierras la aplicación, tu herramienta ya no puede leerlo.', 'Cada cambio que tu herramienta quiera hacer muestra aquí una tarjeta para aprobarlo y pasa a ser un cambio sin guardar que guardas como siempre.', 'La dirección contiene un secreto. Crea una dirección nueva si la compartiste por error. Con un modelo en la nube, lo que lee tu herramienta se envía a ese proveedor.', 'En Claude Code, /plugin install investor --marketplace dobrinov/investor añade habilidades ya preparadas: una revisión mensual, ensayar una operación, la temporada de impuestos y la jubilación.'] },
      { p: 'Cosas que puedes preguntar:' },
      { list: ['«¿Qué tengo y cuánto hay en efectivo?»', '«Compara invertir 10.000 € de golpe con repartirlos en 12 meses, a 20 años vista.»', '«Hoy he vendido 2 VUAA en Xetra a 131,20. Regístralo y dime el impuesto.»', '«¿Voy bien para mis objetivos? ¿Qué haría falta para jubilarme a los 60?»'] },
    ],
  },
  {
    id: 'export-ibkr',
    title: 'Exportar desde Interactive Brokers',
    blocks: [
      { p: 'Investor lee una Activity Flex Query en CSV. La configuras una vez y la ejecutas cuando quieras datos actualizados.' },
      {
        steps: [
          'En el IBKR Portal, abre Performance & Reports → Flex Queries.',
          'Junto a Activity Flex Query pulsa + y llámala «investor-export».',
          'Marca estas secciones y pulsa Select All en cada una: Account Information, Trades (Executions), Cash Transactions, Transaction Fees, Open Positions (Summary), Corporate Actions, Transfers, Financial Instrument Information, Conversion Rates.',
          'Delivery configuration: Format CSV; Include header and trailer records — No; Include column headers — Yes; Include section code and line descriptor — Yes; Date format yyyy-MM-dd; Time format HH:mm:ss; Date/time separator «;».',
          'Guarda y pulsa Run. Elige Custom Date Range desde tu primer ingreso hasta hoy. Si IBKR limita el periodo a 365 días, ejecútala una vez por año.',
          'Importa aquí el archivo descargado. Volver a ejecutarla más adelante es seguro: las filas ya importadas se omiten.',
        ],
      },
      { p: 'Si solo exportas las posiciones abiertas, llegan como saldos iniciales sin fechas de compra ni comisiones. Si después importas el historial completo, se sustituyen automáticamente.' },
    ],
  },
  {
    id: 'export-revolut',
    title: 'Exportar desde Revolut',
    blocks: [
      {
        steps: [
          'En la aplicación de Revolut, abre Invest (o Stocks).',
          'Abre ⋯ o Documents → Brokerage account → Account statement.',
          'Elige Excel/CSV y todo el periodo, y después importa aquí el archivo.',
        ],
      },
      { p: 'Los extractos de Revolut no dicen en qué mercado se ejecutó cada operación. En Bulgaria eso decide si una ganancia está exenta, así que esas ventas se marcan con «Revisa el mercado» en la pantalla de Impuestos.' },
    ],
  },
  {
    id: 'add-broker',
    title: '¿Falta tu bróker o tu mercado?',
    blocks: [
      { p: 'Cada bróker se describe en un pequeño archivo de configuración, así que añadir uno no requiere programar. Hay dos formas.' },
      { p: 'Pídelo. Abre una incidencia en GitHub con el nombre del bróker y una exportación anonimizada:' },
      {
        steps: [
          'Exporta un extracto que tenga al menos un ingreso, una compra, una venta, un dividendo y una comisión, si los tienes.',
          'Sustituye tu número de cuenta, tu nombre y los importes por valores inventados. Mantén los nombres de las columnas, el formato de fecha y las etiquetas de tipo exactamente como se exportaron: es lo que lee la configuración.',
          'Abre una incidencia nueva en GitHub y adjunta el archivo.',
        ],
      },
      { link: 'Abrir una incidencia en GitHub', href: 'https://github.com/dobrinov/investor/issues/new' },
      { p: 'Añádelo tú. Escribe un archivo de configuración como este, ponlo en la carpeta brokers/ del proyecto y abre un pull request. Las filas que tu configuración no reconoce se listan con su número de línea durante la importación, así ves qué falta todavía.' },
      { code: brokerExample },
      { p: 'Tipos de filas: buy, sell, deposit, withdrawal, cash_transfer, dividend, withholding_tax, fee, interest, fx_pair_trade, position_snapshot, ignore. Tipos de comisiones: commission, fx_conversion, exchange, transaction_tax, custody, interest, other.' },
    ],
  },
  {
    id: 'add-tax',
    title: '¿Falta tu administración tributaria?',
    blocks: [
      { p: 'Las normas fiscales van en paquetes: un archivo por país y ejercicio, en la carpeta jurisdictions/ del proyecto (por ejemplo, jurisdictions/bg/2026.toml). Un paquete fija los tipos, las exenciones, los modelos y los plazos.' },
      {
        steps: [
          'Abre una incidencia en GitHub indicando el país y el ejercicio, con un enlace a las instrucciones oficiales de la administración tributaria.',
          'O copia el paquete búlgaro, cambia los valores y abre un pull request.',
          'Deja verified = false hasta que un asesor fiscal haya revisado el paquete. La aplicación muestra un aviso con los paquetes sin verificar.',
        ],
      },
      { code: packExample },
      { p: 'Los valores de arriba son de ejemplo. Cada tipo, exención y nombre de modelo debe salir de las fuentes oficiales.' },
      { link: 'Abrir una incidencia en GitHub', href: 'https://github.com/dobrinov/investor/issues/new' },
    ],
  },
  {
    id: 'glossary',
    title: 'Términos de la aplicación',
    blocks: [
      {
        terms: [
          ['Contabilidad por partida doble', 'Cada movimiento de dinero se anota dos veces, de dónde viene y adónde va, así que la contabilidad siempre cuadra.'],
          ['Asiento', 'Un hecho, como una compra o un dividendo, registrado en apuntes que suman cero.'],
          ['Balance de sumas y saldos', 'El saldo de cada cuenta. Si la contabilidad es correcta, el debe es igual al haber.'],
          ['Coste de adquisición', 'Lo que pagaste por una inversión, incluidas las comisiones cuando las normas lo permiten.'],
          ['FIFO', 'Primero en entrar, primero en salir: al vender, se considera que se venden primero las unidades más antiguas.'],
          ['Ganancia realizada', 'Ganancia o pérdida que se consolida al vender. Es lo que tributa.'],
          ['Ganancia latente', 'Ganancia o pérdida de las inversiones que aún tienes, a su último precio.'],
          ['TER', 'Gastos corrientes totales: el coste anual de un fondo, que se descuenta de su precio.'],
          ['Retención en origen', 'Impuesto que cobra otro país antes de que te llegue un dividendo. A menudo se puede deducir en parte en tu país.'],
          ['Centro de negociación', 'El mercado donde se ejecutó una operación, como Xetra o NASDAQ.'],
          ['ETF', 'Un fondo que cotiza en bolsa como una acción y que suele tener muchas empresas a la vez.'],
          ['Fondo de emergencia', 'Dinero en efectivo para imprevistos, para no tener que vender inversiones en un mal momento.'],
        ],
      },
    ],
  },
]
