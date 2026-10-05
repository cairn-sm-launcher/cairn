// L'ecran Parcourir les serveurs.
//
// Le faux pont rejoue ici le contrat des DEUX commandes, `serveurs` et
// `vue_serveurs`, en refaisant cote navigateur ce que fait
// `cairn_core::serveurs::vue`. C'est une duplication assumee et limitee
// au banc d'essai : l'interet est justement de verifier que l'ecran
// dessine correctement une vue, quelle qu'en soit la provenance. La
// logique reelle, elle, est verifiee par `cargo test`.

import { chromium } from 'playwright';
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';

const RACINE = new URL('../dist', import.meta.url).pathname;
const SORTIE = process.env.SORTIE ?? new URL('./captures', import.meta.url).pathname;
fs.mkdirSync(SORTIE, { recursive: true });

const TYPES = {
  '.html': 'text/html',
  '.js': 'text/javascript',
  '.css': 'text/css',
  '.woff2': 'font/woff2',
  '.svg': 'image/svg+xml',
};
const serveur = http.createServer((req, res) => {
  let p = decodeURIComponent(req.url.split('?')[0]);
  if (p === '/') p = '/index.html';
  const f = path.join(RACINE, p);
  if (!f.startsWith(RACINE) || !fs.existsSync(f)) return res.writeHead(404).end('non');
  res.writeHead(200, { 'content-type': TYPES[path.extname(f)] ?? 'application/octet-stream' });
  res.end(fs.readFileSync(f));
});
await new Promise((r) => serveur.listen(4176, r));

// Un echantillon qui ressemble a ce que rend vraiment l'API : quelques
// serveurs peuples, une majorite de deserts, un prive, un lobby, un
// relais, des noms longs et accentues.
// Le huitieme champ porte les particularites. `segments` remplace le
// decoupage que Rust ferait a partir du nom brut : le banc d'essai ne
// rejoue pas l'analyseur de codes ni le calcul de contraste, qui sont
// couverts par `cargo test` sur les 4096 couleurs possibles. Ce qu'on
// verifie ici, c'est que l'ecran DESSINE correctement un decoupage.
const SERVEURS = [
  ['siege_channel_1', 'Channel: SIEGE EVERYTHING ALLOWED 1', 'SiegePro', 7, 40, 4, 'France', {
    segments: [
      { texte: 'Channel: ', couleur: null, gras: false, italique: false },
      { texte: 'SIEGE', couleur: '#FF6B6B', gras: true, italique: false },
      { texte: ' EVERYTHING ALLOWED 1', couleur: '#7FA6FF', gras: false, italique: true },
    ],
  }],
  ['speedball_pub', 'Speedball #Public', 'SpeedBall', 6, 6, 0, 'Germany', {}],
  ['atria_stretch', 'Λtría - Stretching', 'Obstacle', 1, 30, 0, 'France', {}],
  ['elite_lobby', 'Elite Matchmaking', 'Elite', 3, 200, 0, 'France', { lobby: true }],
  ['nac_bedroom', "Nac's Bedroom", 'Obstacle', 0, 32, 2, 'Belgium', {}],
  ['prive_clan', 'Clan Training', 'Elite', 0, 16, 0, 'France', { prive: true }],
  ['relais_1', 'Atria - Siege 100k (relay)', 'SiegePro', 0, 40, 0, 'France', { relais: 'siege_channel_1' }],
  ['atria_begin', 'Λtría - Beginner', 'Obstacle', 0, 32, 0, 'France', {
    ladder: [0, 50000],
    // Un nom dont la couleur d'origine etait `$001`, invisible sur notre
    // fond, et que Rust a remontee. Ce qu'on regarde ici c'est que la
    // valeur remontee arrive bien jusqu'au style de l'element.
    segments: [{ texte: 'Λtría - Beginner', couleur: '#6E6EFF', gras: false, italique: false }],
  }],
  ['yox_joust', 'Yox々 Joust', 'Joust', 0, 8, 0, 'Japan', {}],
  // Ce serveur n'a AUCUN decoupage, comme en rendrait un binaire Rust
  // d'avant le lot 20. Son nom doit quand meme s'afficher.
  ['royal_pub', 'General Royal Race', 'Royal', 0, 100, 0, '', { sansSegments: true }],
  // Trois modes communautaires de plus : c'est ce qui fait passer la
  // barre de filtres au dela des six puces, donc ce qui met le repli
  // « +N autres » du lot 19d a l'epreuve.
  ['combo_fr', 'Combo France', 'Combo', 0, 24, 0, 'France', {}],
  ['heroes_eu', 'Heroes Arena', 'Heroes', 0, 16, 0, 'Poland', {}],
  ['infect_pub', 'Infection Public', 'Infection', 0, 40, 0, 'France', {}],
];

const PONT = `
window.__SERVEURS__ = ${JSON.stringify(
  SERVEURS.map(([login, nom, mode, joueurs, maximum, observateurs, region, extra]) => ({
    nom,
    nom_brut: nom,
    ...(extra.sansSegments
      ? {}
      : {
          nom_segments: extra.segments ?? [
            { texte: nom, couleur: null, gras: false, italique: false },
          ],
        }),
    mode,
    login,
    titre: 'obstacle@smokegun',
    joueurs,
    maximum,
    observateurs,
    latence: null,
    description: 'Serveur communautaire, tous niveaux bienvenus.',
    region,
    zone: region ? 'World|Europe|' + region : '',
    prive: !!extra.prive,
    observation_privee: false,
    lobby: !!extra.lobby,
    relais: extra.relais ?? null,
    ladder: extra.ladder ?? null,
  })),
)};

window.__FAVORIS__ = window.__FAVORIS__ || [];

window.__ECRITS_REGLAGES__ = null;

window.__REGLAGES_COURANTS__ = function () {
  return Object.assign({
    langue: 'en', accent: '#20E3B2', fond_palette: 'storm',
    fond_intensite: 'medium', fond_vitesse: 'slow',
    fond_personnalise: ['#20E3B2', '#5B8CFF', '#FF6B4A'],
    animations: true, couleurs_jeu: true, densite: 'equilibre',
    fermeture: 'demander', demarrage_auto: false,
    ordre_tuiles: ['favorites', 'popular', 'browse'],
    ordre_tuiles_editer: ['maps', 'replays', 'packs'],
    profil: 'Xirdrak', chemin_jeu: null, assistant_fait: true,
    touche_precedent: 'KeyQ', touche_suivant: 'KeyE', souris_laterale: true,
    themes: [{ nom: '', apparence: null }, { nom: '', apparence: null }, { nom: '', apparence: null }],
    serveurs_mode: '', serveurs_tri: 'joueurs', serveurs_tri_croissant: false,
    serveurs_mot_de_passe: 'tous', serveurs_favoris: false,
    serveurs_peuples_ouverts: true, serveurs_vides_ouverts: false
  }, window.__REGLAGES__ || {}, window.__ECRITS_REGLAGES__ || {},
     { favoris: window.__FAVORIS__.slice() });
};

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
    var tous = window.__SERVEURS__;
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
        return Promise.resolve(window.__REGLAGES_COURANTS__());
      case 'enregistrer_reglages':
        // Rust rend les reglages ASSAINIS, donc un objet. Un retour vide
        // ferait echouer appliquerRetour et poserait une banniere
        // d'erreur sur toutes les captures.
        window.__ECRITS_REGLAGES__ = JSON.parse(JSON.stringify(args.valeurs || {}));
        return Promise.resolve(window.__REGLAGES_COURANTS__());
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
      case 'basculer_favori': {
        var i = window.__FAVORIS__.indexOf(args.login);
        if (i === -1) window.__FAVORIS__.push(args.login); else window.__FAVORIS__.splice(i, 1);
        return Promise.resolve(window.__REGLAGES_COURANTS__());
      }
      case 'etat':
        return Promise.resolve({ chemin_jeu: 'C:/x.exe', jeu_lance: false,
          profils: [{ id:'Xirdrak', nom:'Xirdrak', derniere_utilisation:'2026-09-24',
                      teinte:'#20E3B2', initiale:'X' }], profil_actif: 'Xirdrak' });
      case 'serveurs':
        return Promise.resolve(tous);
      case 'vue_serveurs': {
        var norm = function (v) { return String(v).toLowerCase()
          .replace(/[\\u00e0-\\u00e5\\u03bb\\u03b1]/g, 'a')
          .replace(/[\\u00e8-\\u00eb]/g, 'e')
          .replace(/[\\u00ec-\\u00ef]/g, 'i')
          .replace(/[\\u00f2-\\u00f6]/g, 'o')
          .replace(/[\\u00f9-\\u00fc]/g, 'u'); };
        var mots = String(args.recherche || '').split(/\\s+/).filter(Boolean).map(norm);
        // La recherche porte aussi sur le mode depuis le lot 19d.
        var recherche = function (s) {
          var botte = [norm(s.nom), norm(s.login), norm(s.mode)];
          return mots.every(function (m) { return botte.some(function (c) { return c.indexOf(m) >= 0; }); });
        };
        var prive = args.motDePasse === 'libres' ? false : args.motDePasse === 'prives' ? true : null;
        var passe = function (s) {
          if (args.favorisSeuls && window.__FAVORIS__.indexOf(s.login) === -1) return false;
          if (prive !== null && s.prive !== prive) return false;
          if (args.mode && s.mode !== args.mode) return false;
          return recherche(s);
        };
        var retenus = tous.filter(passe).slice();
        if (args.tri === 'nom') retenus.sort(function (a, b) { return norm(a.nom) < norm(b.nom) ? -1 : 1; });
        else if (args.tri === 'mode') retenus.sort(function (a, b) { return norm(a.mode) < norm(b.mode) ? -1 : norm(a.mode) > norm(b.mode) ? 1 : b.joueurs - a.joueurs; });
        else retenus.sort(function (a, b) { return b.joueurs - a.joueurs; });
        // Le sens : chaque critere a son ordre naturel, l'autre sens le renverse.
        var naturel = args.tri !== 'joueurs';
        if (args.croissant !== naturel) retenus.reverse();
        var comptes = [];
        tous.filter(recherche).forEach(function (s) {
          var c = comptes.find(function (x) { return x.mode === s.mode; });
          if (c) c.compte++; else comptes.push({ mode: s.mode, compte: 1 });
        });
        // Du mode le plus joue au moins joue, alphabetique a egalite.
        comptes.sort(function (a, b) {
          return b.compte - a.compte || (norm(a.mode) < norm(b.mode) ? -1 : 1);
        });
        return Promise.resolve({
          peuples: retenus.filter(function (s) { return s.joueurs > 0 || s.observateurs > 0; }),
          vides: retenus.filter(function (s) { return s.joueurs === 0 && s.observateurs === 0; }),
          comptes: comptes, total: tous.length, retenus: retenus.length
        });
      }
      default:
        return Promise.resolve();
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
  await page.goto('http://127.0.0.1:4176/', { waitUntil: 'networkidle' });
  await page.waitForTimeout(500);
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
  // On arrive par la tuile Browse servers, comme un joueur.
  await page.locator('.tile').nth(2).click();
  await page.waitForTimeout(500);
  return page;
}

const lignes = (page) =>
  page.evaluate(() =>
    [...document.querySelectorAll('[data-serveur]')].map((n) => n.dataset.serveur),
  );

// Deplie les groupes encore replies, sans toucher a ceux qui sont deja
// ouverts. Un `.click()` aveugle sur le deuxieme groupe refermait celui
// qu'une etape precedente avait ouvert, ce qui rendait les lignes
// invisibles et faisait echouer l'etape suivante pour une mauvaise
// raison.
async function deplierTout(page) {
  for (const g of await page.locator('.groupe').all()) {
    if ((await g.locator('.tourne').count()) === 0) await g.click();
  }
  await page.waitForTimeout(250);
}

// 1. L'etat d'ouverture : peuples deplies, vides replies
{
  const page = await ouvrir();
  const visibles = await lignes(page);
  const groupes = await page.evaluate(() =>
    [...document.querySelectorAll('.groupe')].map((b) => b.textContent.trim()),
  );
  console.log('a l ouverture, groupes :', JSON.stringify(groupes));
  console.log('  lignes visibles :', JSON.stringify(visibles));
  await page.screenshot({ path: `${SORTIE}/serveurs-ouverture.png` });

  // On deplie les vides
  await page.locator('.groupe').nth(1).click();
  await page.waitForTimeout(250);
  const apres = await lignes(page);
  console.log('  apres depliage des vides :', apres.length, 'lignes');
  // L attenuation des lignes vides, et sa levee au survol
  const opacites = await page.evaluate(() => {
    const vide = document.querySelector('.bloc.vide .ligne');
    return vide ? getComputedStyle(vide).opacity : null;
  });
  console.log('  opacite d une ligne vide au repos :', opacites);
  await page.screenshot({ path: `${SORTIE}/serveurs-deplies.png` });
  await page.close();
}

// 2. Les filtres comptes, la recherche, le tri
{
  const page = await ouvrir();
  const puces = () =>
    page.evaluate(() =>
      [...document.querySelectorAll('.chip')].map((b) => b.textContent.replace(/\s+/g, ' ').trim()),
    );
  console.log('puces au repos :', JSON.stringify(await puces()));

  await page.locator('input[type="search"]').fill('atria');
  await page.waitForTimeout(350);
  console.log('  puces avec la recherche "atria" :', JSON.stringify(await puces()));
  console.log('  sous-titre :', await page.locator('.compte').textContent());
  await page.screenshot({ path: `${SORTIE}/serveurs-recherche.png` });

  // La recherche ignore les accents : « atria » trouve « Λtría »
  await page.locator('.groupe').nth(1).click().catch(() => {});
  await page.waitForTimeout(250);
  console.log('  lignes trouvees :', JSON.stringify(await lignes(page)));

  await page.locator('input[type="search"]').fill('');
  await page.waitForTimeout(350);
  await page.locator('.seg', { hasText: 'Name' }).click();
  await page.waitForTimeout(300);
  console.log('  tri par nom :', JSON.stringify(await lignes(page)));
  await page.close();
}

// 3. Le tirage au sort
{
  const page = await ouvrir();
  const tirages = [];
  for (let i = 0; i < 4; i++) {
    await page.locator('.hasard').click();
    await page.waitForTimeout(420);
    tirages.push(
      await page.evaluate(() => document.querySelector('.bloc.marquee')?.closest('[data-serveur]')?.dataset.serveur ?? null),
    );
  }
  console.log('tirages successifs :', JSON.stringify(tirages));
  const libelle = await page.locator('.hasard').textContent();
  console.log('  libelle du bouton :', libelle.replace(/\s+/g, ' ').trim());
  const ouvertes = await page.evaluate(() => document.querySelectorAll('.bloc.ouverte').length);
  console.log('  lignes ouvertes en meme temps :', ouvertes);
  await page.screenshot({ path: `${SORTIE}/serveurs-tirage.png` });
  await page.close();
}

// 4. L etat « aucun resultat »
{
  const page = await ouvrir();
  await page.locator('input[type="search"]').fill('zzzzz');
  await page.waitForTimeout(350);
  const etat = await page.locator('.etat-liste').textContent();
  console.log('recherche sans resultat :', etat.replace(/\s+/g, ' ').trim());
  await page.screenshot({ path: `${SORTIE}/serveurs-vide.png` });
  await page.close();
}

// 4 bis. Les rangees tiennent sur une seule ligne a toutes les largeurs
//
// Un controle ne sur un vrai defaut : une regle @media mal fermee avait
// laisse la colonne region visible alors que la grille etait passee a
// cinq colonnes. Les deux enfants en trop repartaient sur une seconde
// rangee implicite, et la hauteur passait de 37 a 66 px. La hauteur est
// donc ce qu'on mesure : elle dit d'un chiffre que rien n'a debord\u00e9.
{
  for (const largeur of [1920, 1440, 1280, 1100, 1024, 900, 860]) {
    const page = await ouvrir(largeur, 800);
    const m = await page.evaluate(() => {
      const ligne = document.querySelector('.ligne');
      const r = ligne.getBoundingClientRect();
      const visibles = [...ligne.children].filter(
        (e) => getComputedStyle(e).display !== 'none',
      ).length;
      const pistes = getComputedStyle(ligne).gridTemplateColumns.split(' ').length;
      return { hauteur: +r.height.toFixed(0), visibles, pistes };
    });
    const ok = m.hauteur <= 40 && m.visibles === m.pistes;
    console.log(
      `  ${largeur} px : hauteur ${m.hauteur} px, ${m.visibles} enfants pour ${m.pistes} pistes`,
      ok ? '' : '  <-- A CORRIGER',
    );
    await page.close();
  }
}

// 5. En francais, et en fenetre etroite
{
  const page = await ouvrir(1100, 800, "window.__REGLAGES__ = { langue: 'fr' };");
  const mesure = await page.evaluate(() => {
    const ligne = document.querySelector('.ligne');
    const st = getComputedStyle(ligne);
    const r = ligne.getBoundingClientRect();
    return {
      largeurFenetre: window.innerWidth,
      requeteActive: matchMedia('(max-width: 1100px)').matches,
      colonnes: st.gridTemplateColumns,
      hauteurRangee: +r.height.toFixed(1),
      largeurRangee: +r.width.toFixed(1),
      region: getComputedStyle(document.querySelector('.region')).display,
      debordement: +(ligne.scrollWidth - ligne.clientWidth).toFixed(1),
    };
  });
  console.log('a 1100 px :', JSON.stringify(mesure));
  await page.screenshot({ path: `${SORTIE}/serveurs-francais-1100.png` });
  await page.close();
}


// 6. Les puces de mode : les six plus joues, le reste derriere « +N autres »
//
// La communaute cree ses propres modes. Certains soirs il y en a une
// quinzaine, et la barre de filtres prenait trois rangees : c'est le
// premier point du retour de Flow au lot 19d.
{
  const page = await ouvrir();
  const puces = () =>
    page.evaluate(() =>
      [...document.querySelectorAll('.chip')].map((b) => b.textContent.replace(/\s+/g, ' ').trim()),
    );
  const avant = await puces();
  console.log('puces repliees :', JSON.stringify(avant));
  await page.locator('.deplier').click();
  await page.waitForTimeout(200);
  console.log('  puces depliees :', (await puces()).length, 'puces');
  await page.screenshot({ path: `${SORTIE}/serveurs-modes-deplies.png` });
  await page.close();
}

// 7. Le sens du tri, et son etat neutre
//
// Deuxieme point du retour : on pouvait trier de A a Z mais pas de Z a
// A. Et au lot 19d bis, Flow a releve qu'il manquait le troisieme etat,
// celui qui ramene la liste dans son ordre d'origine. Le cycle est donc
// sens naturel, sens inverse, retour au tri par defaut.
{
  const page = await ouvrir();
  const actif = () =>
    page.evaluate(
      () => document.querySelector('.tri .seg.actif')?.textContent.trim() ?? null,
    );
  const depart = await lignes(page);
  console.log('tri au depart :', await actif());

  await page.locator('.tri .seg', { hasText: 'Name' }).click();
  await page.waitForTimeout(300);
  const croissant = await lignes(page);
  console.log('  un clic sur Name :', await actif(), JSON.stringify(croissant.slice(0, 3)));

  await page.locator('.tri .seg', { hasText: 'Name' }).click();
  await page.waitForTimeout(300);
  const decroissant = await lignes(page);
  console.log('  deux clics :', await actif(), JSON.stringify(decroissant.slice(0, 3)));
  console.log(
    '  renverse :',
    JSON.stringify(croissant) === JSON.stringify(decroissant.slice().reverse())
      ? 'oui'
      : 'NON  <-- A CORRIGER',
  );

  await page.locator('.tri .seg', { hasText: 'Name' }).click();
  await page.waitForTimeout(300);
  const neutre = await lignes(page);
  console.log('  trois clics :', await actif());
  console.log(
    '  retour a l ordre de depart :',
    JSON.stringify(neutre) === JSON.stringify(depart) ? 'oui' : 'NON  <-- A CORRIGER',
  );

  // Et sur le critere par defaut, le cycle se referme sur lui meme.
  const joueurs = page.locator('.tri .seg', { hasText: 'Players' });
  await joueurs.click();
  await page.waitForTimeout(300);
  const inverse = await lignes(page);
  await joueurs.click();
  await page.waitForTimeout(300);
  console.log(
    '  Players : un clic renverse',
    JSON.stringify(inverse) !== JSON.stringify(depart) ? 'oui' : 'NON',
    '/ deux clics reviennent',
    JSON.stringify(await lignes(page)) === JSON.stringify(depart) ? 'oui' : 'NON',
  );
  await page.close();
}

// 8. Le filtre mot de passe
//
// Troisieme point : il n'existait pas, et un serveur protege se
// decouvrait au moment ou le jeu reclamait un mot de passe.
{
  const page = await ouvrir();
  await page.locator('.mdp .seg', { hasText: 'With password' }).click();
  await deplierTout(page);
  console.log('avec mot de passe :', JSON.stringify(await lignes(page)));
  await page.screenshot({ path: `${SORTIE}/serveurs-prives.png` });

  await page.locator('.mdp .seg', { hasText: 'No password' }).click();
  await deplierTout(page);
  const libres = await lignes(page);
  console.log('  sans mot de passe :', libres.length, 'lignes, dont le prive :',
    libres.includes('prive_clan') ? 'OUI  <-- A CORRIGER' : 'non');
  await page.close();
}

// 9. Les favoris
//
// Cinquieme point, et le plus demande : pouvoir en ajouter, en retirer,
// et les retrouver. Un favori doit se voir plus qu'un serveur a mot de
// passe, d'ou l'etoile pleine et ambree, et il n'est jamais attenue
// meme dans le groupe des serveurs vides.
{
  const page = await ouvrir();
  // On etoile un serveur du groupe des vides, pour verifier du meme coup
  // qu'il cesse d'etre attenue.
  await deplierTout(page);
  await page.locator('[data-serveur="yox_joust"] .etoile').click();
  await page.waitForTimeout(350);

  const etat = await page.evaluate(() => {
    const bloc = document.querySelector('[data-serveur="yox_joust"] .bloc');
    const autre = document.querySelector('[data-serveur="royal_pub"] .bloc');
    return {
      favori: bloc.classList.contains('favori'),
      opaciteFavori: getComputedStyle(bloc.querySelector('.ligne')).opacity,
      opaciteVoisin: getComputedStyle(autre.querySelector('.ligne')).opacity,
      etoilePleine: bloc.querySelector('.etoile svg').getAttribute('fill') !== 'none',
    };
  });
  console.log('apres un clic sur l etoile :', JSON.stringify(etat));
  await page.screenshot({ path: `${SORTIE}/serveurs-favori.png` });

  // Le filtre ne garde que lui, et la puce porte le compte.
  await page.locator('.chip.favoris').click();
  await page.waitForTimeout(350);
  await deplierTout(page);
  const seuls = await page.evaluate(() =>
    [...document.querySelectorAll('[data-serveur]')].map((n) => n.dataset.serveur),
  );
  console.log('  filtre Favorites :', JSON.stringify(seuls));
  console.log('  compte sur la puce :',
    (await page.locator('.chip.favoris').textContent()).replace(/\s+/g, ' ').trim());

  // Et on le retire.
  await page.locator('.chip.favoris').click();
  await page.waitForTimeout(350);
  await deplierTout(page);
  await page.locator('[data-serveur="yox_joust"] .etoile').click();
  await page.waitForTimeout(350);
  const reste = await page.evaluate(() =>
    document.querySelector('[data-serveur="yox_joust"] .bloc').classList.contains('favori'),
  );
  console.log('  encore favori apres retrait :', reste ? 'OUI  <-- A CORRIGER' : 'non');
  await page.close();
}

// 10. La colonne d'etat
//
// Le cadenas etait colle au nom, donc il se deplacait d'une ligne a
// l'autre. Il est maintenant dans une colonne fixe, et ce qu'on verifie
// ici c'est justement qu'il est a la meme abscisse sur toutes les lignes.
{
  const page = await ouvrir();
  await deplierTout(page);
  const abscisses = await page.evaluate(() =>
    [...document.querySelectorAll('.bloc .etat')].map((e) =>
      +e.getBoundingClientRect().left.toFixed(1),
    ),
  );
  const uniques = [...new Set(abscisses)];
  console.log('colonne d etat, abscisses distinctes :', JSON.stringify(uniques));
  console.log('  alignee :', uniques.length === 1 ? 'oui' : 'NON  <-- A CORRIGER');
  await page.close();
}

// 11. Les couleurs des noms
//
// Lot 20. Les noms portent les codes de Trackmania, et leur proprietaire
// les a choisis en regardant le jeu, pas Cairn. Le decoupage et la
// remontee de contraste sont faits en Rust et couverts par `cargo test`
// sur les 4096 couleurs que `$RGB` peut ecrire. Ce qu'on verifie ici,
// c'est que l'ecran dessine ce decoupage : une couleur par morceau, le
// gras et l'italique, et surtout **pas de rangee plus haute pour
// autant**.
{
  const page = await ouvrir();
  const morceaux = await page.evaluate(() =>
    [...document.querySelectorAll('[data-serveur="siege_channel_1"] .texte span')].map(
      (n) => {
        const st = getComputedStyle(n);
        return { texte: n.textContent, couleur: st.color, gras: st.fontWeight, penche: st.fontStyle };
      },
    ),
  );
  console.log('le nom decoupe :', JSON.stringify(morceaux));

  const infobulle = await page.evaluate(
    () => document.querySelector('[data-serveur="siege_channel_1"] .texte').title,
  );
  console.log('  infobulle, nom nu :', JSON.stringify(infobulle));

  // Une rangee coloree et une rangee nue doivent faire la meme hauteur.
  await deplierTout(page);
  const hauteurs = await page.evaluate(() => {
    const h = (login) =>
      +document
        .querySelector(`[data-serveur="${login}"] .ligne`)
        .getBoundingClientRect()
        .height.toFixed(1);
    return { coloree: h('siege_channel_1'), nue: h('speedball_pub') };
  });
  console.log(
    '  hauteur coloree contre nue :',
    JSON.stringify(hauteurs),
    hauteurs.coloree === hauteurs.nue ? '' : '  <-- A CORRIGER',
  );
  await page.screenshot({ path: `${SORTIE}/serveurs-couleurs.png` });
  await page.close();
}

// 12. Le nom s'affiche toujours
//
// **Ce controle est ne d'un vrai defaut, et d'un defaut silencieux.**
// Chez Flow, un binaire Rust d'avant le lot 20 tournait avec l'interface
// d'apres : le champ `nom_segments` n'existait pas, `{#each}` sur un
// champ absent ne rend rien et ne se plaint pas, et **toute la colonne
// des noms s'est vidée** sans une ligne en console. Un nom est ce qui
// identifie une ligne, donc il doit s'afficher quoi qu'il arrive.
//
// L'echantillon porte exprès un serveur sans decoupage.
{
  const page = await ouvrir();
  await deplierTout(page);
  const nom = (login) =>
    page.evaluate(
      (l) => document.querySelector(`[data-serveur="${l}"] .texte`).textContent.trim(),
      login,
    );
  const sansDecoupage = await nom('royal_pub');
  console.log('un serveur sans decoupage affiche :', JSON.stringify(sansDecoupage));
  console.log(
    '  nom visible :',
    sansDecoupage.length > 0 ? 'oui' : 'NON  <-- A CORRIGER',
  );

  const avant = await page.evaluate(
    () => document.querySelectorAll('[data-serveur="siege_channel_1"] .texte span').length,
  );
  await page.close();

  // Et le reglage qui coupe les couleurs rend le nom nu, sans rien
  // perdre. Une page neuve et pas un rechargement : le faux pont est
  // pose par addInitScript, donc un rechargement effacerait le reglage
  // qu'on vient d'ecrire dans la page.
  const eteint = await ouvrir(1440, 900, 'window.__REGLAGES__ = { couleurs_jeu: false };');
  const apres = await eteint.evaluate(() => ({
    morceaux: document.querySelectorAll('[data-serveur="siege_channel_1"] .texte span')
      .length,
    texte: document
      .querySelector('[data-serveur="siege_channel_1"] .texte')
      .textContent.trim(),
  }));
  console.log(`couleurs coupees : ${avant} morceaux avant, ${apres.morceaux} apres`);
  console.log('  le nom reste entier :', JSON.stringify(apres.texte));
  await eteint.screenshot({ path: `${SORTIE}/serveurs-sans-couleurs.png` });
  await eteint.close();
}

console.log('erreurs console :', erreurs.length ? erreurs : 'aucune');
await navigateur.close();
serveur.close();
