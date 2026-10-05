// Releve toutes les phrases passees a t() dans les composants et dit
// lesquelles manquent au dictionnaire francais.
//
// Une phrase absente ne casse rien, elle s'affiche en anglais. C'est
// justement pour ca qu'il faut un controle : sans lui, un oubli passe
// inapercu pendant des semaines sur un poste en anglais.

import fs from 'node:fs';
import path from 'node:path';

const RACINE = new URL('../src', import.meta.url).pathname;

function fichiers(dossier) {
  return fs.readdirSync(dossier, { withFileTypes: true }).flatMap((e) => {
    const p = path.join(dossier, e.name);
    if (e.isDirectory()) return fichiers(p);
    return /\.(svelte|ts)$/.test(e.name) ? [p] : [];
  });
}

const fr = fs.readFileSync(path.join(RACINE, 'lib/fr.ts'), 'utf8');
const connues = new Set(
  [...fr.matchAll(/^\s{2}("(?:[^"\\]|\\.)*"):/gm)].map((m) => JSON.parse(m[1])),
);

// Les tournures a trous vivent dans MOTIFS, pas dans fr.ts.
const traduction = fs.readFileSync(path.join(RACINE, 'lib/traduction.svelte.ts'), 'utf8');
const motifs = new Set(
  [...traduction.matchAll(/^\s{2}('(?:[^'\\]|\\.)*'):/gm)].map((m) =>
    m[1].slice(1, -1).replace(/\\'/g, "'"),
  ),
);

const manquantes = new Map();
for (const f of fichiers(RACINE)) {
  if (f.endsWith('lib/fr.ts') || f.endsWith('traduction.svelte.ts')) continue;
  const texte = fs.readFileSync(f, 'utf8');
  // t('...') et t("...") sur une ou plusieurs lignes, y compris avec des
  // apostrophes echappees.
  for (const m of texte.matchAll(/\bt\(\s*(['"])((?:[^\\]|\\.)*?)\1/gs)) {
    const phrase = m[2].replace(/\\'/g, "'").replace(/\\n\s*/g, ' ').trim();
    if (!phrase) continue;
    if (connues.has(phrase) || motifs.has(phrase)) continue;
    if (!manquantes.has(phrase)) manquantes.set(phrase, new Set());
    manquantes.get(phrase).add(path.relative(RACINE, f));
  }
}

if (manquantes.size === 0) {
  console.log('Traduction : aucune phrase manquante.');
} else {
  console.log(`Traduction : ${manquantes.size} phrase(s) sans equivalent francais.\n`);
  for (const [phrase, fichiers] of manquantes) {
    console.log(`  ${JSON.stringify(phrase)}`);
    console.log(`      vu dans ${[...fichiers].join(', ')}`);
  }
  process.exitCode = 1;
}
