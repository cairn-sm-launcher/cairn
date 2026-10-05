// Le repli des etiquettes de touches, quand le systeme ne sait pas
// donner la disposition reelle.
//
// C'est le pendant de `cairn_core::touches::etiquette_de_repli`, et le
// seul endroit de l'interface ou une regle du coeur metier est
// dupliquee. La duplication est assumee pour une raison precise :
// l'etiquette est demandee a chaque rendu d'une pastille de raccourci,
// donc un aller-retour vers Rust la rendrait asynchrone pour afficher
// une seule lettre. La regle, elle, ne bouge pas : elle suppose un
// QWERTY et ne sert que le temps que le systeme reponde.
//
// La version de reference reste celle de Rust, testee. Si l'une des
// deux change, c'est celle-ci qui suit.

const SYMBOLES: Record<string, string> = {
  ArrowUp: '↑',
  ArrowDown: '↓',
  ArrowLeft: '←',
  ArrowRight: '→',
  Comma: ',',
  Period: '.',
  Slash: '/',
  Semicolon: ';',
  Quote: "'",
  Backquote: '`',
  Minus: '-',
  Equal: '=',
  BracketLeft: '[',
  BracketRight: ']',
  Backslash: '\\',
  IntlBackslash: '<',
};

export function repliEtiquette(code: string): string {
  if (code.startsWith('Key')) return code.slice(3);
  if (code.startsWith('Digit')) return code.slice(5);
  if (code.startsWith('Numpad')) return `№${code.slice(6)}`;
  return SYMBOLES[code] ?? code;
}
