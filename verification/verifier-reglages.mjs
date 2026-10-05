// L'ecran Reglages : ses deux sous-onglets, la capture de raccourci, la
// reorganisation des tuiles, les emplacements de themes.
//
// Le faux pont rejoue ici la partie de `cairn_core::reglages::assainir`
// qui se voit a l'ecran : le refus des touches reservees et le retour
// des reglages retenus. Le reste de l'assainissement est verifie par
// `cargo test`, ce qu'on regarde ici c'est que l'interface montre bien
// ce que Rust a garde et pas ce qu'elle esperait.

import { chromium } from 'playwright';
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';

const RACINE = new URL('../dist', import.meta.url).pathname;
const SORTIE = process.env.SORTIE ?? new URL('./captures', import.meta.url).pathname;
fs.mkdirSync(SORTIE, { recursive: true });

const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.woff2': 'font/woff2' };
const serveur = http.createServer((q, r) => {
  let p = decodeURIComponent(q.url.split('?')[0]);
  if (p === '/') p = '/index.html';
  const f = path.join(RACINE, p);
  if (!f.startsWith(RACINE) || !fs.existsSync(f)) return r.writeHead(404).end('non');
  r.writeHead(200, { 'content-type': TYPES[path.extname(f)] ?? 'application/octet-stream' });
  r.end(fs.readFileSync(f));
});
await new Promise((r) => serveur.listen(4178, r));

const PONT = `
window.__ETAT__ = {
  langue: 'en', accent: '#20E3B2', fond_palette: 'storm',
  fond_intensite: 'medium', fond_vitesse: 'slow',
  fond_personnalise: ['#20E3B2', '#5B8CFF', '#FF6B4A'],
  animations: true, couleurs_jeu: true, densite: 'equilibre',
  fermeture: 'demander', demarrage_auto: false,
  ordre_tuiles: ['favorites', 'popular', 'browse'],
  ordre_tuiles_editer: ['maps', 'replays', 'packs'],
  profil: 'Xirdrak', chemin_jeu: null, assistant_fait: true,
  touche_precedent: 'KeyQ', touche_suivant: 'KeyE', souris_laterale: true,
  themes: [{nom:'',apparence:null},{nom:'',apparence:null},{nom:'',apparence:null}],
  serveurs_mode: '', serveurs_tri: 'joueurs', serveurs_tri_croissant: false,
  serveurs_mot_de_passe: 'tous', serveurs_favoris: false, favoris: [],
  serveurs_peuples_ouverts: true, serveurs_vides_ouverts: false
};
window.__ECRITS__ = [];

// Ce que Rust refuse comme raccourci, rejoue ici.
var RESERVEES = ['Tab','Enter','NumpadEnter','Escape','Space','ShiftLeft','ShiftRight',
                 'ControlLeft','ControlRight','AltLeft','AltRight','MetaLeft'];
function codeValide(c) {
  if (RESERVEES.indexOf(c) >= 0) return false;
  return /^Key[A-Z]$/.test(c) || /^Digit[0-9]$/.test(c) || /^Numpad[0-9]$/.test(c)
    || ['ArrowUp','ArrowDown','ArrowLeft','ArrowRight','Comma','Period','Slash'].indexOf(c) >= 0;
}

window.__HISTO__ = { entrees: ['play'], position: 0 };
window.__ETAT_NAV__ = function () {
  var h = window.__HISTO__;
  return {
    ecran: h.entrees[h.position],
    peut_reculer: h.position > 0,
    peut_avancer: h.position + 1 < h.entrees.length
  };
};

window.__RAPPELS__ = {};

window.__TAURI_INTERNALS__ = {
  // Ce que l'API d'evenements de Tauri appelle pour enregistrer un
  // rappel. Sans lui, l'ecoute echoue et la question de la croix ne peut
  // pas etre mise a l'epreuve.
  transformCallback: function (rappel) {
    var id = Object.keys(window.__RAPPELS__).length + 1;
    window.__RAPPELS__[id] = rappel;
    return id;
  },
  invoke: function (cmd, args) {
    switch (cmd) {
      case 'glisser_tuile': {
        if (args.grille !== 'jouer' && args.grille !== 'editer') {
          return Promise.resolve(args.ordre.slice());
        }
        // Le banc d'essai ne rejoue PAS la regle : il l'applique par le
        // meme raisonnement, en quelques lignes, pour que le controle
        // porte sur ce que l'ecran fait de la reponse. La regle elle
        // meme est couverte par cargo test.
        var o = args.ordre.slice();
        var i = o.indexOf(args.depuis);
        var j = o.indexOf(args.vers);
        if (i < 0 || j < 0 || args.depuis === args.vers) return Promise.resolve(o);
        o.splice(i, 1);
        o.splice(Math.min(j, o.length), 0, args.depuis);
        return Promise.resolve(o);
      }
      case 'decaler_tuile': {
        if (args.grille !== 'jouer' && args.grille !== 'editer') {
          return Promise.resolve(args.ordre.slice());
        }
        var od = args.ordre.slice();
        var id = od.indexOf(args.id);
        var jd = id + args.pas;
        if (id >= 0 && jd >= 0 && jd < od.length) {
          od[id] = od[jd];
          od[jd] = args.id;
        }
        return Promise.resolve(od);
      }
      case 'plugin:event|listen': {
        window.__ECOUTES__ = window.__ECOUTES__ || {};
        window.__ECOUTES__[args.event] = window.__RAPPELS__[args.handler];
        return Promise.resolve(1);
      }
      case 'plugin:event|unlisten':
        return Promise.resolve();
      case 'fermer_ou_reduire':
        window.__FERMETURE__ = { choix: args.choix, memoriser: args.memoriser };
        return Promise.resolve();
      case 'diagnostic': {
        var diag = {
          nom: 'Cairn', version: '0.1.0', coeur: '0.1.0',
          os: 'windows', arch: 'x86_64', journal: null,
        };
        // __SANS_CONTRAT__ rejoue un moteur d'avant le mecanisme, qui ne
        // rend AUCUN numero. Un repli sur 1 ne suffirait pas a le
        // simuler, d'ou ce drapeau a part.
        if (!window.__SANS_CONTRAT__) diag.contrat = window.__CONTRAT__ ?? 5;
        return Promise.resolve(diag);
      }
      case 'naviguer': {
        var h = window.__HISTO__;
        if (h.entrees[h.position] !== args.ecran) {
          h.entrees = h.entrees.slice(0, h.position + 1);
          h.entrees.push(args.ecran);
          h.position = h.entrees.length - 1;
        }
        return Promise.resolve(window.__ETAT_NAV__());
      }
      case 'reculer': {
        var hr = window.__HISTO__;
        if (hr.position > 0) hr.position--;
        return Promise.resolve(window.__ETAT_NAV__());
      }
      case 'avancer': {
        var ha = window.__HISTO__;
        if (ha.position + 1 < ha.entrees.length) ha.position++;
        return Promise.resolve(window.__ETAT_NAV__());
      }
      case 'reglages':
        return Promise.resolve(JSON.parse(JSON.stringify(window.__ETAT__)));
      case 'enregistrer_reglages': {
        var v = JSON.parse(JSON.stringify(args.valeurs));
        if (!codeValide(v.touche_precedent)) v.touche_precedent = 'KeyQ';
        if (!codeValide(v.touche_suivant)) v.touche_suivant = 'KeyE';
        if (v.touche_precedent === v.touche_suivant) {
          v.touche_precedent = 'KeyQ'; v.touche_suivant = 'KeyE';
        }
        window.__ETAT__ = v;
        window.__ECRITS__.push(cmd);
        return Promise.resolve(JSON.parse(JSON.stringify(v)));
      }
      case 'etat':
        return Promise.resolve({ chemin_jeu: 'C:\\\\Program Files (x86)\\\\ManiaPlanet\\\\ManiaPlanet.exe',
          jeu_lance: false,
          profils: [{ id:'Xirdrak', nom:'Xirdrak', derniere_utilisation:'2026-09-24',
                      teinte:'#20E3B2', initiale:'X' }], profil_actif: 'Xirdrak' });
      case 'veille':
        return Promise.resolve({ jeu_lance: false, profils_a_relire: false, jeu_a_change: false });
      case 'tri_suivant': {
        var naturel = function (t) { return t !== 'joueurs'; };
        if (args.courant !== args.demande)
          return Promise.resolve([args.demande, naturel(args.demande)]);
        if (args.croissant === naturel(args.demande))
          return Promise.resolve([args.demande, !naturel(args.demande)]);
        return Promise.resolve(['joueurs', naturel('joueurs')]);
      }
      case 'serveurs': return Promise.resolve([]);
      case 'vue_serveurs':
        return Promise.resolve({ peuples: [], vides: [], comptes: [], total: 0, retenus: 0 });
      default: return Promise.resolve();
    }
  }
};`;

// Le faux pont vit dans un litteral gabarit : un accent grave dans un
// commentaire le fermerait en plein milieu, et tous les controles
// porteraient sur du vide. Ce test le dit tout de suite.
if (!PONT.includes('invoke: function') || !PONT.trimEnd().endsWith('};')) {
  throw new Error('PONT tronque. Cherche un accent grave dedans.');
}


const navigateur = await chromium.launch(
  process.env.CHROMIUM ? { executablePath: process.env.CHROMIUM } : {},
);
const erreurs = [];

async function ouvrir(largeur = 1440, hauteur = 900, avant = '') {
  const page = await navigateur.newPage({ viewport: { width: largeur, height: hauteur } });
  page.on('console', (m) => m.type() === 'error' && erreurs.push(m.text()));
  page.on('pageerror', (e) => erreurs.push(e.message));
  await page.addInitScript(avant + '\n' + PONT);
  await page.goto('http://127.0.0.1:4178/', { waitUntil: 'networkidle' });
  await page.waitForTimeout(500);
  await page.locator('button[title="Launcher settings"]').click();
  await page.waitForTimeout(400);
  // Le faux pont s'est-il bien installe ?
  //
  // Il vit dans un litteral gabarit, donc un simple accent grave dans un
  // commentaire le ferme en plein milieu et `__TAURI_INTERNALS__` n'est
  // jamais defini. L'application part alors sur l'assistant de premier
  // lancement et tous les controles mesurent le vide, sans que rien ne
  // dise pourquoi. C'est arrive deux fois : autant le detecter ici.
  if (!(await page.evaluate(() => '__TAURI_INTERNALS__' in window))) {
    throw new Error(
      'Le faux pont ne s\'est pas installe. Cherche un accent grave dans PONT.',
    );
  }
  return page;
}

const touches = (page) =>
  page.evaluate(() => [...document.querySelectorAll('.touche')].map((t) => t.textContent.trim()));
const pastilles = (page) =>
  page.evaluate(() => [...document.querySelectorAll('.keyhint')].map((k) => k.textContent.trim()));
const retenu = (page) =>
  page.evaluate(() => [window.__ETAT__.touche_precedent, window.__ETAT__.touche_suivant]);

// 1. Les deux sous-onglets
{
  const page = await ouvrir();
  const sections = () =>
    page.evaluate(() => [...document.querySelectorAll('.section')].map((h) => h.textContent.trim()));
  console.log('General  :', JSON.stringify(await sections()));
  await page.screenshot({ path: `${SORTIE}/reglages-general.png`, fullPage: true });
  await page.locator('.seg', { hasText: 'Appearance' }).click();
  await page.waitForTimeout(300);
  console.log('Apparence:', JSON.stringify(await sections()));
  await page.screenshot({ path: `${SORTIE}/reglages-apparence.png`, fullPage: true });
  await page.close();
}

// 2. La capture d'un raccourci
{
  const page = await ouvrir();
  console.log('touches au depart :', JSON.stringify(await touches(page)), await retenu(page));

  // Une touche libre : elle doit etre retenue
  await page.locator('button', { hasText: 'Change' }).first().click();
  await page.waitForTimeout(200);
  await page.keyboard.press('KeyW');
  await page.waitForTimeout(400);
  console.log('  apres KeyW  :', JSON.stringify(await touches(page)), await retenu(page));

  // Une touche reservee : Rust refuse, et on doit retrouver le raccourci
  // qu'on avait, KeyW, et non la valeur par defaut.
  await page.locator('button', { hasText: 'Change' }).first().click();
  await page.waitForTimeout(200);
  await page.keyboard.press('Tab');
  await page.waitForTimeout(700);
  console.log('  apres Tab   :', JSON.stringify(await touches(page)), await retenu(page));
  const avis = await page.evaluate(() => document.querySelector('.toast')?.textContent?.trim() ?? null);
  console.log('  message     :', JSON.stringify(avis));

  // La touche deja prise par l'autre sens
  await page.locator('button', { hasText: 'Change' }).first().click();
  await page.waitForTimeout(200);
  await page.keyboard.press('KeyE');
  await page.waitForTimeout(400);
  console.log('  apres KeyE  :', JSON.stringify(await touches(page)), await retenu(page));

  // Echap annule sans rien changer
  await page.locator('button', { hasText: 'Change' }).first().click();
  await page.waitForTimeout(200);
  await page.keyboard.press('Escape');
  await page.waitForTimeout(300);
  console.log('  apres Echap :', JSON.stringify(await touches(page)), await retenu(page));
  await page.close();
}

// 3. La navigation ne se declenche pas pendant une capture
{
  const page = await ouvrir();
  // On est sur l'ecran Reglages, donc aucun onglet n'est actif. Ce qu'on
  // verifie, c'est qu'on y est toujours : KeyE est le raccourci « onglet
  // suivant », et pendant une capture il ne doit surtout pas naviguer.
  const surReglages = () =>
    page.evaluate(() => document.querySelector('.colonne .titre')?.textContent?.trim() ?? null);
  await page.locator('button', { hasText: 'Change' }).first().click();
  await page.waitForTimeout(200);
  await page.keyboard.press('KeyE');
  await page.waitForTimeout(400);
  console.log('toujours sur l ecran Reglages apres KeyE :', JSON.stringify(await surReglages()));
  await page.close();
}

// 4. L'ordre des tuiles
{
  const page = await ouvrir();
  const ordre = () => page.evaluate(() => window.__ETAT__.ordre_tuiles);
  console.log('ordre des tuiles :', JSON.stringify(await ordre()));
  await page.locator('button[title="Move down"]').first().click();
  await page.waitForTimeout(600);
  console.log('  apres une descente :', JSON.stringify(await ordre()));
  const desactives = await page.evaluate(() =>
    [...document.querySelectorAll('.fleche')].map((b) => b.disabled),
  );
  console.log('  fleches desactivees aux bouts :', JSON.stringify(desactives));
  await page.close();
}

// 5. Les emplacements de themes
{
  const page = await ouvrir();
  await page.locator('.seg', { hasText: 'Appearance' }).click();
  await page.waitForTimeout(300);

  // On choisit une apparence, on l'enregistre, on change, on recharge.
  await page.locator('.palette', { hasText: 'Ember' }).click();
  await page.waitForTimeout(200);
  await page.locator('.swatch').nth(4).click();
  await page.waitForTimeout(600);
  const choisi = await page.evaluate(() => [window.__ETAT__.accent, window.__ETAT__.fond_palette]);
  await page.locator('.emplacement').first().locator('button', { hasText: 'Save' }).click();
  await page.waitForTimeout(600);

  await page.locator('.palette', { hasText: 'Slate' }).click();
  await page.locator('.swatch').nth(1).click();
  await page.waitForTimeout(600);
  const entre = await page.evaluate(() => [window.__ETAT__.accent, window.__ETAT__.fond_palette]);

  await page.locator('.emplacement').first().locator('button', { hasText: 'Load' }).click();
  await page.waitForTimeout(600);
  const repose = await page.evaluate(() => [window.__ETAT__.accent, window.__ETAT__.fond_palette]);
  console.log('theme enregistre :', JSON.stringify(choisi));
  console.log('  puis change en :', JSON.stringify(entre));
  console.log('  puis recharge  :', JSON.stringify(repose));

  // La remise a zero garde les emplacements
  await page.locator('button', { hasText: 'Reset to default theme' }).click();
  await page.waitForTimeout(250);
  await page.locator('button', { hasText: 'Reset' }).last().click();
  await page.waitForTimeout(600);
  const apresRaz = await page.evaluate(() => ({
    accent: window.__ETAT__.accent,
    palette: window.__ETAT__.fond_palette,
    emplacementGarde: window.__ETAT__.themes[0].apparence !== null,
  }));
  console.log('  apres remise a zero :', JSON.stringify(apresRaz));
  await page.screenshot({ path: `${SORTIE}/reglages-themes.png`, fullPage: true });
  await page.close();
}

// 5 bis. La densite
//
// Trois reglages dans Apparence, et ce qui compte est qu'ils soient
// retenus : le fichier porte la valeur choisie, et la racine porte
// l'attribut que la feuille de style lit.
{
  const page = await ouvrir();
  await page.locator('.seg', { hasText: 'Appearance' }).click();
  await page.waitForTimeout(200);
  const densite = () =>
    page.evaluate(() => ({
      racine: document.querySelector('.root').dataset.densite,
      retenue: window.__ETAT__.densite,
    }));
  console.log('densite au depart :', JSON.stringify(await densite()));
  for (const libelle of ['Compact', 'Airy', 'Balanced']) {
    await page.getByRole('button', { name: libelle, exact: true }).click();
    await page.waitForTimeout(500);
    console.log(`  apres « ${libelle} » :`, JSON.stringify(await densite()));
  }

  // L'apercu, ne du retour de Flow : il doit suivre le SURVOL et pas le
  // choix, sinon il faut valider pour voir, ce qui est exactement le
  // probleme qu'il decrit.
  const mesureApercu = () =>
    page.evaluate(() => {
      const a = document.querySelector('.apercu');
      const st = getComputedStyle(a.querySelector('.tuiles'));
      return {
        montre: a.dataset.densite,
        ecartHaut: st.marginTop,
        gouttiere: st.gap,
      };
    });
  console.log('apercu au repos :', JSON.stringify(await mesureApercu()));
  for (const libelle of ['Compact', 'Airy']) {
    await page.getByRole('button', { name: libelle, exact: true }).hover();
    await page.waitForTimeout(350);
    console.log(`  survol « ${libelle} » :`, JSON.stringify(await mesureApercu()));
  }
  await page.mouse.move(0, 0);
  await page.waitForTimeout(350);
  console.log('  survol quitte :', JSON.stringify(await mesureApercu()));

  // Le bloc des actualites doit tenir dans la boite aux trois densites :
  // `aere` est le cas le plus haut, et un apercu tronque mentirait sur
  // le reglage le plus genereux.
  for (const libelle of ['Compact', 'Balanced', 'Airy']) {
    await page.getByRole('button', { name: libelle, exact: true }).hover();
    await page.waitForTimeout(350);
    const debord = await page.evaluate(() => {
      const a = document.querySelector('.apercu');
      const actus = a.querySelector('.actus');
      return +(
        actus.getBoundingClientRect().bottom -
        (a.getBoundingClientRect().bottom - parseFloat(getComputedStyle(a).paddingBottom))
      ).toFixed(1);
    });
    console.log(
      `  debordement en « ${libelle} » : ${debord} px`,
      debord > 0.5 ? '  <-- A CORRIGER' : '',
    );
  }
  await page.mouse.move(0, 0);
  await page.waitForTimeout(300);
  console.log('  la densite retenue n a pas bouge :', (await densite()).retenue);
  await page.screenshot({ path: `${SORTIE}/reglages-densite.png` });
  await page.close();
}

// 6. L'interrupteur des animations coupe bien le mouvement
{
  const page = await ouvrir();
  const anim = () => page.evaluate(() => document.querySelector('.root').className);
  console.log('racine au depart :', await anim());
  await page.locator('[role="switch"]').last().click();
  await page.waitForTimeout(500);
  console.log('  apres extinction :', await anim());
  await page.close();
}

// 7. En francais
{
  const page = await ouvrir(1440, 900, "window.__LANGUE__='fr';");
  await page.locator('.seg', { hasText: 'Français' }).click();
  await page.waitForTimeout(500);
  const titre = await page.locator('.titre.display').textContent();
  console.log('titre en francais :', titre.trim());
  await page.screenshot({ path: `${SORTIE}/reglages-francais.png`, fullPage: true });
  await page.close();
}

console.log('erreurs console :', erreurs.length ? erreurs : 'aucune');
await navigateur.close();
serveur.close();
