// Les quelques calculs de couleur de l'interface.

const SECOURS = 'rgba(32,227,178,';

/**
 * Une couleur hexadecimale plus une opacite, en `rgba()`.
 *
 * Accepte la forme courte a trois chiffres, et retombe sur le teal Storm
 * quand l'entree n'est pas lisible. Ce secours n'est pas une politesse :
 * la couleur part dans un attribut `style`, donc elle ne doit jamais
 * pouvoir en sortir. Le controle strict, lui, est fait cote Rust sur ce
 * qui vient du disque, voir `cairn_core::reglages::couleur_valide`.
 */
export function rgba(hex: string, alpha: number): string {
  let h = String(hex ?? '').replace('#', '');
  if (h.length === 3) h = h[0] + h[0] + h[1] + h[1] + h[2] + h[2];
  if (h.length !== 6 || !/^[0-9a-f]{6}$/i.test(h)) return SECOURS + alpha + ')';
  const n = Number.parseInt(h, 16);
  return `rgba(${(n >> 16) & 255},${(n >> 8) & 255},${n & 255},${alpha})`;
}
