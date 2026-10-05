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
  if (!f.startsWith(RACINE) || !fs.existsSync(f)) {
    res.writeHead(404).end('non');
    return;
  }
  res.writeHead(200, { 'content-type': TYPES[path.extname(f)] ?? 'application/octet-stream' });
  res.end(fs.readFileSync(f));
});
await new Promise((r) => serveur.listen(4173, r));

// Le faux pont. Il rejoue ce que Rust rendrait sur le poste de Flow, en
// reprenant les vraies valeurs relevees dans son journal au lot 19b.
const PONT = `
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
    window.__APPELS__ = window.__APPELS__ || [];
    window.__APPELS__.push([cmd, args]);
    switch (cmd) {
      case 'glisser_tuile': {
        if (args.grille !== 'jouer' && args.grille !== 'editer') {
          return Promise.resolve(args.ordre.slice());
        }
        // Le banc d'essai ne rejoue PAS la regle : il l'applique par le
        // meme raisonnement, en quelques lignes, pour que le controle
        // porte sur ce que l'ecran fait de la reponse. La regle elle
        // meme est couverte par cargo test.
        //
        // La tuile portee prend la place de celle qu'elle survole : vers
        // la droite on se pose derriere la cible, vers la gauche devant.
        var g = args.ordre.slice();
        var gi = g.indexOf(args.depuis);
        var gj = g.indexOf(args.vers);
        if (gi < 0 || gj < 0 || args.depuis === args.vers) return Promise.resolve(g);
        g.splice(gi, 1);
        g.splice(Math.min(gj, g.length), 0, args.depuis);
        return Promise.resolve(g);
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
      case 'inventaire': {
        // L'echantillon reprend la forme du dossier de Flow : des maps a
        // la racine et dans My Maps, des replays ranges, et des title
        // packs dont l'un porte un nom bricole, donc sans identifiant de
        // titre et sans bouton d'ouverture.
        if (window.__SANS_FICHIERS__) {
          var rien = { nombre: 0, octets: 0, dernier: null, groupes: [], recents: [] };
          return Promise.resolve({ maps: rien, replays: rien, packs: rien });
        }
        return Promise.resolve({
          maps: {
            nombre: 23, octets: 48123456, dernier: 1788369669,
            groupes: [
              { nom: 'My Maps', nombre: 17, octets: 40000000 },
              { nom: '', nombre: 4, octets: 7000000 },
              { nom: 'Downloaded', nombre: 2, octets: 1123456 }
            ],
            recents: [
              { nom: 'BXS_Item_Testing', groupe: '', octets: 2142314, modifie: 1788369669, titre: null },
              { nom: 'BXS', groupe: 'My Maps', octets: 1251907, modifie: 1788109421, titre: null },
              { nom: 'PIXEL - Chinatown', groupe: 'My Maps', octets: 4178088, modifie: 1778499446, titre: null }
            ]
          },
          replays: {
            nombre: 8, octets: 3211000, dernier: 1788100000,
            groupes: [
              { nom: 'Replays', nombre: 6, octets: 2900000 },
              { nom: 'Autosaves', nombre: 2, octets: 311000 }
            ],
            recents: [
              { nom: 'Chinatown_xirdrak', groupe: 'Replays', octets: 410000, modifie: 1788100000, titre: null }
            ]
          },
          packs: {
            nombre: 4, octets: 1680000000, dernier: 1760449376,
            groupes: [{ nom: '', nombre: 4, octets: 1680000000 }],
            recents: [
              { nom: 'bricole', groupe: '', octets: 1000, modifie: 1790000000, titre: null },
              { nom: 'GEs@guerro', groupe: '', octets: 426728392, modifie: 1760449376, titre: 'GEs@guerro' },
              { nom: 'obstacle@smokegun', groupe: '', octets: 1171302920, modifie: 1759674438, titre: 'obstacle@smokegun' },
              { nom: 'SMStormElite@nadeolabs', groupe: '', octets: 37047496, modifie: 1707735009, titre: 'SMStormElite@nadeolabs' }
            ]
          }
        });
      }
      case 'ouvrir_dossier': {
        window.__DOSSIER_OUVERT__ = args.cle;
        return Promise.resolve();
      }
      // Le journal du lot 23. Le faux pont NE REJOUE PAS l'anonymisation
      // ni le decoupage : ils vivent en Rust et y sont testes trente-deux
      // fois. Ce qui se verifie ici est ce que l'INTERFACE demande, d'ou
      // le releve de tous les appels et de leurs arguments.
      case 'lire_journal': {
        window.__APPELS_JOURNAL__ = window.__APPELS_JOURNAL__ || [];
        window.__APPELS_JOURNAL__.push({
          niveaux: args.niveaux.slice().sort(),
          recherche: args.recherche,
          anonyme: args.anonyme,
          limite: args.limite,
        });
        var toutes = window.__LIGNES_JOURNAL__ || [];
        var retenues = toutes.filter(function (l) {
          var bonNiveau = args.niveaux.indexOf(l.niveau) >= 0;
          var mot = (args.recherche || '').trim().toLowerCase();
          return bonNiveau && (mot === '' || l.message.toLowerCase().indexOf(mot) >= 0);
        });
        var tronquees = 0;
        if (args.limite > 0 && retenues.length > args.limite) {
          tronquees = retenues.length - args.limite;
          retenues = retenues.slice(tronquees);
        }
        var comptes = { info: 0, alerte: 0, erreur: 0 };
        toutes.forEach(function (l) { comptes[l.niveau] += 1; });
        return Promise.resolve({
          lignes: retenues,
          comptes: comptes,
          anonyme: args.anonyme,
          tronquees: tronquees,
        });
      }
      // Le fil d'actualites du lot 25. Le faux pont NE REJOUE NI la
      // signature NI le decoupage du markdown : les deux vivent en Rust
      // et y sont testes vingt-neuf fois, signature comprise, contre le
      // vecteur d'essai de la RFC 8032. Ce qui se verifie ici est ce que
      // l'INTERFACE demande et ce qu'elle dessine.
      case 'actualites': {
        window.__APPELS_ACTUS__ = window.__APPELS_ACTUS__ || [];
        window.__APPELS_ACTUS__.push({
          forcer: args.forcer,
          langue: args.langue,
          categories: args.categories.slice().sort(),
          recherche: args.recherche,
        });
        if (window.__ACTUS_EN_PANNE__) {
          return Promise.resolve({
            articles: [], comptes: [], publie: null,
            avertissement: window.__ACTUS_NON_CONFIGUREES__ ? 'nonconfigure' : 'reseau',
          });
        }
        var tous = window.__ARTICLES__ || [];
        var vus = tous.filter(function (a) {
          return a.langue === '' || a.langue === args.langue;
        });
        var comptes = [];
        vus.forEach(function (a) {
          var trouve = false;
          for (var i = 0; i < comptes.length; i++) {
            if (comptes[i][0] === a.categorie) { comptes[i][1] += 1; trouve = true; }
          }
          if (!trouve) comptes.push([a.categorie, 1]);
        });
        var gardes = vus.filter(function (a) {
          var bonneCategorie =
            args.categories.length === 0 || args.categories.indexOf(a.categorie) >= 0;
          var mot = (args.recherche || '').trim().toLowerCase();
          return bonneCategorie &&
            (mot === '' || (a.titre + ' ' + a.resume).toLowerCase().indexOf(mot) >= 0);
        });
        gardes.sort(function (x, y) {
          if (x.epingle !== y.epingle) return x.epingle ? -1 : 1;
          return y.date - x.date;
        });
        return Promise.resolve({
          articles: gardes,
          comptes: comptes,
          publie: 1791100000,
          avertissement: window.__ACTUS_HORS_LIGNE__ ? 'reseau' : null,
        });
      }
      case 'ouvrir_lien': {
        window.__LIENS_OUVERTS__ = window.__LIENS_OUVERTS__ || [];
        window.__LIENS_OUVERTS__.push(args.lien);
        return Promise.resolve();
      }
      case 'rapport_journal': {
        window.__RAPPORTS_DEMANDES__ = (window.__RAPPORTS_DEMANDES__ || 0) + 1;
        // On releve les arguments recus, pour pouvoir verifier qu'il n'y
        // en a AUCUN : le rapport ne doit pas pouvoir suivre le choix
        // d'affichage.
        window.__ARGS_RAPPORT__ = Object.keys(args || {});
        return Promise.resolve('Rapport Cairn, version 0.1.0, contrat 6.');
      }
      case 'enregistrer_rapport':
        window.__RAPPORT_ECRIT__ = true;
        return Promise.resolve('rapport-2026-09-30-120000.txt');
      case 'ouvrir_dossier_journal':
        window.__DOSSIER_JOURNAL_OUVERT__ = true;
        return Promise.resolve();
      case 'ouvrir_titre': {
        window.__TITRE_OUVERT__ = args.titre;
        return Promise.resolve({ relais: false });
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
      case 'diagnostic': {
        var diag = {
          nom: 'Cairn', version: '0.1.0', coeur: '0.1.0',
          os: 'windows', arch: 'x86_64', journal: null,
        };
        // __SANS_CONTRAT__ rejoue un moteur d'avant le mecanisme, qui ne
        // rend AUCUN numero. Un repli sur 1 ne suffirait pas a le
        // simuler, d'ou ce drapeau a part.
        if (!window.__SANS_CONTRAT__) diag.contrat = window.__CONTRAT__ ?? 7;
        return Promise.resolve(diag);
      }
      case 'reglages':
        return Promise.resolve(Object.assign({
          langue: 'en', accent: '#20E3B2', fond_palette: 'storm',
          fond_intensite: 'medium', fond_vitesse: 'slow',
          fond_personnalise: ['#20E3B2', '#5B8CFF', '#FF6B4A'],
          animations: true, couleurs_jeu: true, densite: 'equilibre',
    fermeture: 'demander', demarrage_auto: false,
          ordre_tuiles: ['favorites', 'popular', 'browse'],
          ordre_tuiles_editer: ['maps', 'replays', 'packs'],
          touche_precedent: 'KeyQ', touche_suivant: 'KeyE', souris_laterale: true,
          themes: [{nom:'',apparence:null},{nom:'',apparence:null},{nom:'',apparence:null}],
          serveurs_mode: '', serveurs_tri: 'joueurs', serveurs_tri_croissant: false,
          serveurs_mot_de_passe: 'tous', serveurs_favoris: false, favoris: [],
          serveurs_peuples_ouverts: true, serveurs_vides_ouverts: false,
          profil: 'xirdrak', assistant_fait: true
        }, window.__REGLAGES__ || {}));
      case 'veille':
        return Promise.resolve({ jeu_lance: false, profils_a_relire: false, jeu_a_change: false });
      case 'etat':
        return Promise.resolve({
          chemin_jeu: window.__SANS_JEU__ ? null : 'C:\\\\Program Files (x86)\\\\ManiaPlanet\\\\ManiaPlanet.exe',
          jeu_lance: false,
          profils: [{ id: 'xirdrak', nom: 'Xirdrak', derniere_utilisation: '2026-09-24',
                      teinte: '#20E3B2', initiale: 'X' },
                    { id: 'Xirdrak_test', nom: 'Xirdrak_test', derniere_utilisation: '2026-09-11',
                      teinte: '#FFC857', initiale: 'X' }],
          profil_actif: 'xirdrak'
        });
      case 'serveurs':
        return Promise.resolve([
          { nom: 'Channel: SIEGE EVERYTHING ALLOWED 1', nom_brut: '',
            nom_segments: [{ texte: 'Channel: SIEGE EVERYTHING ALLOWED 1', couleur: null, gras: false, italique: false }],
            mode: 'SiegePro',
            login: 'siege_channel_1', titre: 'siege@smokegun', joueurs: 7, maximum: 40,
            observateurs: 4, latence: null },
          { nom: 'Speedball #Public', nom_brut: '',
            nom_segments: [{ texte: 'Speedball #Public', couleur: null, gras: false, italique: false }],
            mode: 'SpeedBall',
            login: 'speedball_pub', titre: 'speedball@nadeo', joueurs: 6, maximum: 6,
            observateurs: 0, latence: null },
          { nom: 'Atria - Stretching', nom_brut: '',
            nom_segments: [{ texte: 'Atria - Stretching', couleur: null, gras: false, italique: false }],
            mode: 'Obstacle',
            login: 'Atria_Stretching', titre: 'obstacle@smokegun', joueurs: 1, maximum: 30,
            observateurs: 0, latence: null }
        ]);
      case 'enregistrer_reglages':
        // Rust rend les reglages assainis : rendre \`undefined\` ferait
        // echouer appliquerRetour et poserait une banniere d'erreur.
        window.__GARDE__ = args.valeurs;
        return Promise.resolve(args.valeurs);
      default:
        return Promise.reject('commande inconnue : ' + cmd);
    }
  }
};
`;

// Le faux pont vit dans un litteral gabarit : un accent grave dans un
// commentaire le fermerait en plein milieu, et tous les controles
// porteraient sur du vide. Ce test le dit tout de suite.
if (!PONT.includes('invoke: function') || !PONT.trimEnd().endsWith('};')) {
  throw new Error('PONT tronque. Cherche un accent grave dedans.');
}

// Les deux numeros de contrat doivent etre egaux, et AUCUN banc d'essai
// ne peut le voir : le faux pont rend celui qu'on lui donne, donc un
// moteur reste en arriere passerait tous les controles ici et poserait
// une banniere ambre en permanence chez l'utilisateur. La verification
// se fait donc sur les sources, et elle a ete ecrite le jour ou le
// numero a ete monte d'un cote et pas de l'autre.
{
  const rust = fs.readFileSync(new URL('../crates/cairn-core/src/lib.rs', import.meta.url), 'utf8');
  const ts = fs.readFileSync(new URL('../src/lib/pont.ts', import.meta.url), 'utf8');
  const cote = (texte, motif) => texte.match(motif)?.[1];
  const aRust = cote(rust, /pub const CONTRAT: u32 = (\d+);/);
  const aTs = cote(ts, /export const CONTRAT_ATTENDU = (\d+);/);
  if (!aRust || !aTs) throw new Error('Numero de contrat introuvable dans un des deux fichiers.');
  if (aRust !== aTs) {
    throw new Error(
      `Contrats desaccordes : cairn_core::CONTRAT vaut ${aRust}, CONTRAT_ATTENDU vaut ${aTs}.`,
    );
  }
  console.log(`contrat des sources : ${aRust} des deux cotes`);
}


// executablePath n'est utile que dans un conteneur ou Playwright ne gere
// pas lui meme ses navigateurs. Sur un poste ordinaire, laisser vide.
const navigateur = await chromium.launch(
  process.env.CHROMIUM ? { executablePath: process.env.CHROMIUM } : {},
);
const erreurs = [];

async function capturer(nom, largeur, hauteur, avant, options) {
  const page = await navigateur.newPage({
    viewport: { width: largeur, height: hauteur },
    ...options,
  });
  page.on('console', (m) => m.type() === 'error' && erreurs.push(`${nom}: ${m.text()}`));
  page.on('pageerror', (e) => erreurs.push(`${nom}: ${e.message}`));
  await page.addInitScript(avant ? avant + '\n' + PONT : PONT);
  await page.goto('http://127.0.0.1:4173/', { waitUntil: 'networkidle' });
  await page.waitForTimeout(700);
  // Le faux pont s'est-il bien installe ?
  //
  // Il vit dans un litteral gabarit, donc un simple accent grave dans un
  // commentaire le ferme en plein milieu et `__TAURI_INTERNALS__` n'est
  // jamais defini. L'application part alors sur l'assistant de premier
  // lancement et tous les controles mesurent le vide, sans que rien ne
  // dise pourquoi. C'est arrive deux fois : autant le detecter ici.
  if (!(await page.evaluate(() => '__TAURI_INTERNALS__' in window))) {
    throw new Error(
      'Le faux pont ne s\'est pas installe. PONT est un litteral gabarit, donc ' +
        'deux choses le cassent : un accent grave non echappe, et une sequence ' +
        "d'echappement comme \\n, que le gabarit consomme avant JavaScript et " +
        'qui coupe la chaine generee en deux. Les deux sont arrivees.',
    );
  }
  await page.screenshot({ path: `${SORTIE}/${nom}.png`, fullPage: false });
  return page;
}

// 1. L'application, aux trois largeurs du lot 17
const p1440 = await capturer('jouer-1440', 1440, 900);
const p1920 = await capturer('jouer-1920', 1920, 1080);
const p1100 = await capturer('jouer-1100', 1100, 800);

// Le centrage des onglets, mesure et pas estime
for (const [nom, page] of [['1440', p1440], ['1920', p1920], ['1100', p1100]]) {
  const m = await page.evaluate(() => {
    const nav = document.querySelector('.tab3-row');
    if (!nav) return null;
    const r = nav.getBoundingClientRect();
    const bandeau = document.querySelector('.bandeau').getBoundingClientRect();
    // L'etat du jeu est reduit a sa pastille depuis l'etape 30. Ce qu'on
    // verifie n'est donc plus que le libelle s'affiche, mais qu'il reste
    // joignable : l'infobulle doit porter la phrase entiere, sinon
    // l'information a disparu au lieu d'avoir ete repliee.
    const etat = document.querySelector('.gamestate');
    // Et que le groupe de droite ne vienne pas mordre sur les onglets :
    // c'est ce qui s'est produit quand le profil y est arrive. On mesure
    // le premier ENFANT et non la boite du groupe : celle-ci est en
    // flex 1 1 0, donc elle occupe toujours toute sa part et son bord
    // gauche vaut invariablement la gouttiere, ce qui ne dit rien de
    // l'air reellement visible. Mesure faite : 18 px annonces contre
    // 110 px reels.
    return {
      ecart: +(r.left + r.width / 2 - (bandeau.left + bandeau.width / 2)).toFixed(2),
      hauteurBandeau: +bandeau.height.toFixed(1),
      etatJoignable: !!etat?.getAttribute('title'),
      airADroite: (() => {
        const premier = document.querySelector('.head3-right')?.firstElementChild;
        return premier ? +(premier.getBoundingClientRect().left - r.right).toFixed(1) : null;
      })(),
      premiereTuile: +document.querySelector('.tile').getBoundingClientRect().top.toFixed(1),
      // La position de la premiere tuile depend de la densite depuis le
      // lot 19d. Ces trois captures tournent a la densite par defaut,
      // donc `equilibre` : le chiffre du lot 17, 102 px, se lit dans le
      // controle des densites plus bas, en `compact`.
      densite: document.querySelector('.root').dataset.densite,
    };
  });
  console.log(nom, JSON.stringify(m));
}

// 2. L'assistant de premier lancement, ses trois etapes
const pa = await capturer('assistant-1', 1440, 900, 'window.__REGLAGES__ = { assistant_fait: false };');
const hauteurs = [];
for (const etape of [1, 2, 3]) {
  if (etape > 1) {
    await pa.getByRole('button', { name: /Continue|Finish setup/ }).click();
    await pa.waitForTimeout(350);
  }
  hauteurs.push(
    await pa.evaluate(() => +document.querySelector('.carte').getBoundingClientRect().height.toFixed(1)),
  );
  await pa.screenshot({ path: `${SORTIE}/assistant-${etape}.png` });
}
console.log('hauteur de la carte aux trois etapes:', hauteurs.join(' / '));

// 1 bis. La colonne plafonnee et les gouttieres progressives
// La gouttiere interne suit la densite depuis le lot 19d. On mesure
// donc en `compact`, qui est la mise en page de reference du lot 17 :
// c'est ce qui rend ces chiffres comparables d'un lot a l'autre.
for (const largeur of [1100, 1280, 1440, 1920, 2560]) {
  const page = await capturer(
    `colonne-${largeur}`,
    largeur,
    900,
    "window.__REGLAGES__ = { densite: 'compact' };",
  );
  const m = await page.evaluate(() => {
    const pad = document.querySelector('.pad');
    const r = pad.getBoundingClientRect();
    const st = getComputedStyle(pad);
    const grille = document.querySelector('.grille');
    return {
      colonne: +r.width.toFixed(1),
      centree: +(r.left - (window.innerWidth - r.width) / 2).toFixed(1),
      gouttiereExterne: st.paddingLeft,
      gouttiereInterne: grille ? getComputedStyle(grille).gap : null,
    };
  });
  console.log('colonne', largeur, JSON.stringify(m));
  await page.close();
}

// 1 ter. Le rebond quand la navigation bute, et les pastilles eteintes
//
// **On appuie sur q et e, pas sur a et e.** Depuis le lot 19c-3 le
// raccourci porte sur la POSITION de la touche, `KeyQ`, et non sur la
// lettre. C'est la meme position physique qui porte A sur le clavier de
// Flow et Q sur le clavier QWERTY que simule Playwright. Ecrire 'a' ici
// enverrait `KeyA`, qui n'est affecte a rien.
//
// **Methode, et pourquoi celle la.** En sans-tete, l'horloge d'animation
// n'avance que quand une image est produite, et ni un releve en boucle
// serree ni une capture n'en produisent assez : on mesure alors toujours
// currentTime = 0 et on conclut a tort que rien ne bouge. Deux essais y
// sont passes. On ne mesure donc pas le rendu, on **pilote la ligne de
// temps** : on place l'animation a l'instant voulu et on lit la valeur
// qui en sort. C'est deterministe, et c'est ce qui compte ici, la
// fluidite reelle n'etant de toute facon pas observable en sans-tete.
{
  const page = await capturer('rebond', 1440, 900);

  const lire = () =>
    page.evaluate(() => ({
      ecran: document.querySelector('.tab3.on span')?.textContent?.trim(),
      eteintes: [...document.querySelectorAll('.keyhint')].map((k) =>
        k.className.includes('eteinte'),
      ),
    }));

  // Rend la course du rebond : le deplacement a chaque dixieme de sa
  // duree, plus son amplitude et sa duree totale.
  const course = () =>
    page.evaluate(() => {
      const nav = document.querySelector('.tab3-row');
      // Une animation creee par element.animate() n'est pas une
      // CSSAnimation : elle n'a pas de animationName, donc on ne peut
      // pas la reconnaitre par son nom. La rangee n'en porte aucune
      // autre, on prend la seule qui s'y trouve.
      const a = nav.getAnimations()[0];
      if (!a) return null;
      const duree = a.effect.getComputedTiming().duration;
      const x = [];
      for (let p = 0; p <= 1.0001; p += 0.1) {
        a.currentTime = duree * p;
        x.push(+new DOMMatrixReadOnly(getComputedStyle(nav).transform).m41.toFixed(2));
      }
      return { duree, x };
    });

  // Cinq onglets depuis l'etape 30, et on demarre sur Jouer, qui est au
  // milieu. Il faut donc deux pas pour atteindre chaque bord, la ou
  // l'ancien bandeau a trois onglets mettait Jouer au bord gauche.
  const onglets = await page.evaluate(() =>
    [...document.querySelectorAll('.tab3 span:first-child')].map((s) => s.textContent.trim()),
  );
  console.log('onglets :', JSON.stringify(onglets));
  console.log('ecran de depart :', (await lire()).ecran);

  // Deux pas vers la gauche pour arriver au bord, le troisieme rebondit.
  await page.keyboard.press('q');
  await page.waitForTimeout(320);
  await page.keyboard.press('q');
  await page.waitForTimeout(320);
  const bordGauche = await lire();
  await page.keyboard.press('q');
  await page.waitForTimeout(40);
  const gauche = await course();
  await page.waitForTimeout(400);
  const apres = await lire();
  console.log(
    `bute a gauche depuis ${bordGauche.ecran} : ${gauche?.duree} ms,`,
    `course ${JSON.stringify(gauche?.x)},`,
    `ecran apres ${apres.ecran}, pastilles eteintes ${JSON.stringify(bordGauche.eteintes)}`,
  );

  // Puis quatre pas vers la droite pour l'autre bord.
  for (let i = 0; i < 4; i++) {
    await page.keyboard.press('e');
    await page.waitForTimeout(320);
  }
  const bout = await lire();
  await page.keyboard.press('e');
  await page.waitForTimeout(40);
  const droite = await course();
  console.log(
    `bute a droite depuis ${bout.ecran} : ${droite?.duree} ms,`,
    `course ${JSON.stringify(droite?.x)},`,
    `pastilles eteintes ${JSON.stringify(bout.eteintes)}`,
  );

  // Une navigation qui aboutit ne doit pas rebondir.
  await page.waitForTimeout(400);
  await page.keyboard.press('q');
  await page.waitForTimeout(60);
  const inutile = await course();
  console.log(`navigation qui aboutit : ${inutile === null ? 'aucun rebond' : 'REBOND A TORT'}`);
  await page.close();
}

// 4 ter. L'accuse de reception quand le rebond se tait
//
// Le rebond obeit a l'interrupteur des animations et au reglage systeme
// de reduction de mouvement. Quand il se tait, la touche ne faisait plus
// rien du tout, ce qui est exactement la panne apparente qu'il existait
// pour eviter. La pastille du bout s'allume donc aussi, et c'est ce
// controle la qui le garantit dans les trois reglages.
{
  const essais = [
    ['mouvement normal', undefined, undefined],
    ['mouvement reduit', undefined, { reducedMotion: 'reduce' }],
    ['animations coupees', 'window.__REGLAGES__ = { animations: false };', undefined],
  ];
  for (const [nom, avant, options] of essais) {
    const page = await capturer(`butee-${nom.split(' ')[1]}`, 1440, 900, avant, options);
    // Trois pas vers la gauche depuis Jouer : deux qui aboutissent, un
    // qui bute.
    for (let i = 0; i < 3; i++) {
      await page.keyboard.press('q');
      await page.waitForTimeout(i < 2 ? 320 : 60);
    }
    const vu = await page.evaluate(() => {
      const nav = document.querySelector('.tab3-row');
      const gauche = document.querySelector('.keyhint');
      return {
        ecran: document.querySelector('.root').dataset.ecran,
        rebond: nav.getAnimations().length > 0,
        pastilleAllumee: gauche.className.includes('heurtee'),
        opacite: +getComputedStyle(gauche).opacity.slice(0, 4),
      };
    });
    // Puis elle doit s'eteindre, faute de quoi elle resterait allumee pour
    // de bon. On lit la CLASSE et non l'opacite : celle-ci repasse par une
    // transition, donc un releve tombe au mauvais moment rend une valeur
    // intermediaire et le controle changerait d'avis d'une execution a
    // l'autre. La classe, elle, part a coup sur au bout des 420 ms. Le
    // piege a coute un aller-retour, il est note pour la prochaine fois.
    await page.waitForTimeout(900);
    const apres = await page.evaluate(() =>
      document.querySelector('.keyhint').className.includes('heurtee'),
    );
    console.log(
      `butee, ${nom} :`,
      JSON.stringify({ ...vu, encoreAllumeeApres: apres }),
      vu.pastilleAllumee && !apres ? '' : ' <-- ACCUSE DE RECEPTION A CORRIGER',
    );
    await page.close();
  }
}

// 2 bis. La saisie manuelle du chemin, et le cas ou le jeu n'est pas trouve
const psans = await capturer(
  'assistant-sans-jeu',
  1440,
  900,
  'window.__REGLAGES__ = { assistant_fait: false }; window.__SANS_JEU__ = true;',
);
await psans.screenshot({ path: `${SORTIE}/assistant-sans-jeu.png` });
await psans.getByRole('button', { name: /Set the path myself/ }).click();
await psans.waitForTimeout(300);
await psans.screenshot({ path: `${SORTIE}/assistant-saisie.png` });

// 3. Le selecteur de profil
await pa.getByRole('button', { name: /Finish setup/ }).click().catch(() => {});
await pa.waitForTimeout(400);
await pa.locator('.profile-btn').click();
await pa.waitForTimeout(300);
await pa.screenshot({ path: `${SORTIE}/selecteur.png` });

// 4. La tuile Popular ouverte, details compris
const pt = await capturer('tuile-popular', 1440, 900);
await pt.locator('.tile').nth(1).click();
await pt.waitForTimeout(200);
await pt.locator('.helpdot').click();
await pt.waitForTimeout(400);
await pt.screenshot({ path: `${SORTIE}/tuile-popular.png` });

// 5. Le francais
const pf = await capturer('francais', 1440, 900, "window.__REGLAGES__ = { langue: 'fr' };");
await pf.screenshot({ path: `${SORTIE}/francais.png` });

// 6. Les trois densites
//
// Flow a demande du blanc entre le bandeau, les tuiles et les
// actualites, calibre sur la hauteur du bandeau. Ce qu'on mesure ici,
// c'est que `compact` rend EXACTEMENT la mise en page d'avant le lot
// 19d, celle du lot 17 : premiere tuile a 102 px du haut. Les deux
// autres reglages montent, et l'ecart entre les tuiles et les
// actualites suit le meme mouvement.
{
  console.log('densites :');
  for (const densite of ['compact', 'equilibre', 'aere']) {
    const page = await capturer(
      `densite-${densite}`,
      1440,
      900,
      `window.__REGLAGES__ = { densite: '${densite}' };`,
    );
    const m = await page.evaluate(() => {
      const tuile = document.querySelector('.tile').getBoundingClientRect();
      const grille = document.querySelector('.grille').getBoundingClientRect();
      const actus = document.querySelector('.section');
      return {
        premiereTuile: +tuile.top.toFixed(0),
        tuilesActualites: actus
          ? +(actus.getBoundingClientRect().top - grille.bottom).toFixed(0)
          : null,
      };
    });
    const note =
      densite === 'compact' && m.premiereTuile !== 102 ? '  <-- compact a bouge' : '';
    console.log(`  ${densite} : ${JSON.stringify(m)}${note}`);
    await page.close();
  }
}

// 7. L'historique de navigation
//
// Ne du retour de Flow : les boutons lateraux de la souris faisaient
// exactement ce que font les deux touches, donc ils ne servaient a
// rien. Ils remontent maintenant le chemin reellement parcouru.
//
// **Les boutons 3 et 4 de la souris ne se simulent pas** avec
// `page.mouse`, qui ne connait que gauche, milieu et droite. On envoie
// donc l'evenement que le navigateur enverrait, ce qui teste bien notre
// gestionnaire meme si ca ne teste pas le pilote de souris.
{
  const page = await capturer('navigation', 1440, 900);
  // L'ecran courant se lit sur la racine du document. Le deviner a
  // partir de l'onglet allume ne marche pas : Reglages et Parcourir les
  // serveurs n'en allument aucun.
  const ecran = () =>
    page.evaluate(() => document.querySelector('.root').dataset.ecran);
  const fleche = () =>
    page.evaluate(() => {
      const b = document.querySelector('.head3-left button');
      return b ? { eteinte: b.disabled, opacite: getComputedStyle(b).opacity } : null;
    });
  const souris = (bouton) =>
    page.evaluate(
      (b) => window.dispatchEvent(new MouseEvent('mouseup', { button: b, bubbles: true })),
      bouton,
    );

  console.log('fleche au demarrage :', JSON.stringify(await fleche()));

  // Jouer, puis Reglages par la roue dentee, puis l'onglet Historique.
  await page.locator('.head3-right button[aria-label="Launcher settings"]').click();
  await page.waitForTimeout(250);
  const apresReglages = await ecran();
  await page.locator('.tab3', { hasText: 'History' }).click();
  await page.waitForTimeout(250);
  console.log('chemin parcouru :', JSON.stringify(['Play', apresReglages, await ecran()]));
  console.log('  fleche apres navigation :', JSON.stringify(await fleche()));

  // Le bouton Precedent de la souris remonte ce chemin la, et pas la
  // rangee d'onglets : il doit ramener sur Reglages et non sur Jouer.
  await souris(3);
  await page.waitForTimeout(250);
  console.log('  un Precedent souris :', await ecran());
  await souris(3);
  await page.waitForTimeout(250);
  console.log('  deux Precedent :', await ecran());
  await souris(4);
  await page.waitForTimeout(250);
  console.log('  un Suivant :', await ecran());

  // Alt et les fleches font la meme chose, pour les souris sans boutons
  // lateraux.
  await page.keyboard.press('Alt+ArrowLeft');
  await page.waitForTimeout(250);
  console.log('  Alt et fleche gauche :', await ecran());

  // Les deux touches de navigation empilent elles aussi : ce sont des
  // changements d'ecran comme les autres.
  await page.keyboard.press('e');
  await page.waitForTimeout(250);
  const apresTouche = await ecran();
  await souris(3);
  await page.waitForTimeout(250);
  console.log(`  apres la touche Suivant (${apresTouche}), un retour donne :`, await ecran());

  // Un sous-onglet de Reglages n'empile pas : c'est une commande dans un
  // ecran, pas un ecran. Sinon Precedent deviendrait « annuler ».
  await page.locator('.head3-right button[aria-label="Launcher settings"]').click();
  await page.waitForTimeout(250);
  await page.locator('.seg', { hasText: 'Appearance' }).click();
  await page.waitForTimeout(250);
  await souris(3);
  await page.waitForTimeout(250);
  console.log('  retour depuis un sous-onglet de Reglages :', await ecran());

  // Et au bout de la pile, la fleche s'eteint au lieu de disparaitre.
  for (let i = 0; i < 8; i++) {
    await souris(3);
    await page.waitForTimeout(120);
  }
  console.log('  au bout de la pile :', await ecran(), JSON.stringify(await fleche()));
  await page.screenshot({ path: `${SORTIE}/navigation.png` });
  await page.close();
}

// 8. Le moteur et l'interface ne sont pas de la meme version
//
// **Ne d'une soiree perdue.** Le lot 20 a ajoute un champ au serveur que
// le pont rend. Vite a recharge l'interface a chaud, le binaire Rust
// non, et l'application a tourne avec une interface qui attendait un
// champ qu'un moteur plus ancien ne rendait pas : colonne des noms vide,
// puis sans couleurs, et **pas un signal**. Le binaire avait ete
// compile quatre-vingt-neuf minutes avant les sources.
//
// Le numero de contrat rend ce cas bruyant, et ce controle verifie
// qu'il le reste.
{
  const normal = await capturer('contrat-ok', 1440, 900);
  console.log(
    'contrat en accord, message :',
    await normal.evaluate(() => document.querySelector('.depareille')?.textContent.trim() ?? 'aucun'),
  );
  await normal.close();

  // Un moteur qui rend un autre numero.
  const vieux = await capturer('contrat-vieux', 1440, 900, 'window.__CONTRAT__ = 0;');
  console.log(
    'contrat en desaccord :',
    await vieux.evaluate(() => {
      const n = document.querySelector('.depareille');
      return n ? n.textContent.trim().slice(0, 60) + '...' : 'AUCUN MESSAGE  <-- A CORRIGER';
    }),
  );
  await vieux.screenshot({ path: `${SORTIE}/contrat-vieux.png` });
  await vieux.close();

  // Et un moteur d'avant le mecanisme, qui ne rend aucun numero : c'est
  // le cas reel, et il doit compter comme un desaccord.
  const absent = await capturer('contrat-absent', 1440, 900, 'window.__SANS_CONTRAT__ = true;');
  console.log(
    'contrat absent :',
    await absent.evaluate(() =>
      document.querySelector('.depareille') ? 'prevenu' : 'AUCUN MESSAGE  <-- A CORRIGER',
    ),
  );
  await absent.close();
}

// 9. La croix de la fenetre
//
// Rust empeche la fermeture et passe la main a l'interface, qui pose la
// question. Trois precautions a tenir, et chacune repare une facon
// connue de pieger les gens : la case est decochee, le bouton qui ferme
// n'a pas le focus, et Echap annule sans rien faire.
{
  const page = await capturer('fermeture', 1440, 900);
  const demander = () =>
    page.evaluate(() => window.__ECOUTES__['fermeture-demandee']({ payload: null }));
  const etat = () =>
    page.evaluate(() => {
      const carte = document.querySelector('[role="dialog"]');
      if (!carte) return null;
      const focus = document.activeElement;
      return {
        case: carte.querySelector('input[type="checkbox"]').checked,
        focus: focus ? focus.textContent.trim() : null,
        boutons: [...carte.querySelectorAll('button')].map((b) => b.textContent.trim()),
      };
    });

  await demander();
  await page.waitForTimeout(250);
  console.log('la croix pose la question :', JSON.stringify(await etat()));
  await page.screenshot({ path: `${SORTIE}/fermeture.png` });

  // Echap annule : ni fermer, ni reduire, ni reponse envoyee.
  await page.keyboard.press('Escape');
  await page.waitForTimeout(200);
  console.log(
    '  apres Echap, carte :',
    (await etat()) === null ? 'fermee' : 'ENCORE LA  <-- A CORRIGER',
    '/ reponse envoyee :',
    await page.evaluate(() => (window.__FERMETURE__ ? 'OUI  <-- A CORRIGER' : 'aucune')),
  );

  // Reduire sans cocher : la reponse part, le choix n'est pas retenu.
  await demander();
  await page.waitForTimeout(250);
  await page.getByRole('button', { name: 'Minimize', exact: true }).click();
  await page.waitForTimeout(250);
  console.log(
    '  Reduire sans cocher :',
    JSON.stringify(await page.evaluate(() => window.__FERMETURE__)),
  );

  // Et en cochant, le choix est demande en memoire.
  await demander();
  await page.waitForTimeout(250);
  await page.locator('[role="dialog"] input[type="checkbox"]').check();
  await page.getByRole('button', { name: 'Close', exact: true }).click();
  await page.waitForTimeout(250);
  console.log(
    '  Fermer en cochant :',
    JSON.stringify(await page.evaluate(() => window.__FERMETURE__)),
  );
  await page.close();
}

// 10. Ranger les tuiles
//
// Deux chemins, la souris et le clavier, et une seule regle en Rust. Ce
// qu'on verifie ici est que l'ecran envoie le bon mouvement et affiche
// le resultat, pas que la regle est juste : celle-la est couverte par
// `cargo test`.
//
// Le glisser passe aux evenements de pointeur au lot 22c, et ce banc
// d'essai en profite : Playwright pilote un vrai pointeur, donc il
// exerce maintenant le geste lui meme et plus des evenements fabriques a
// la main. La version d'avant envoyait des DragEvent avec un
// DataTransfer reel, ce qui verifiait nos gestionnaires sans jamais
// toucher la couche de glisser du navigateur. C'est precisement ce
// qu'elle ne voyait pas qui a casse chez Flow.
{
  const page = await capturer('tuiles', 1440, 900);
  const ordre = () =>
    page.evaluate(() =>
      [...document.querySelectorAll('.grille .libelle')].map((n) => n.textContent.trim()),
    );
  /** Le centre d'une case, par son rang dans la grille. */
  const centre = (rang) =>
    page.evaluate((rang) => {
      const boite = document.querySelectorAll('.case')[rang].getBoundingClientRect();
      return { x: boite.left + boite.width / 2, y: boite.top + boite.height / 2 };
    }, rang);
  const etat = () =>
    page.evaluate(() => ({
      creux: document.querySelectorAll('.creux').length,
      porteur: !!document.querySelector('.porteur'),
      grilleEnGlisse: !!document.querySelector('.grille.enGlisse'),
    }));

  console.log('ordre au depart :', JSON.stringify(await ordre()));

  // Le clavier : Ctrl et une fleche sur la tuile qui a le focus.
  await page.locator('.case').first().locator('.tile').focus();
  await page.keyboard.press('Control+ArrowRight');
  await page.waitForTimeout(300);
  console.log('  apres Ctrl et fleche droite :', JSON.stringify(await ordre()));
  console.log(
    '  la tuile deplacee garde le focus :',
    await page.evaluate(
      () => document.activeElement?.querySelector('.libelle')?.textContent.trim() ?? '?',
    ),
  );
  await page.keyboard.press('Control+ArrowLeft');
  await page.waitForTimeout(300);
  console.log('  et retour :', JSON.stringify(await ordre()));

  // Au bord, rien ne bouge et surtout rien ne boucle.
  await page.keyboard.press('Control+ArrowLeft');
  await page.waitForTimeout(300);
  console.log('  au bord gauche :', JSON.stringify(await ordre()));

  // Le maintien, sans bouger la souris d'un pixel : la tuile se souleve
  // quand meme. C'est ce qui rend le geste decouvrable, on appuie et on
  // voit que ca bouge.
  const un = await centre(0);
  await page.mouse.move(un.x, un.y);
  await page.mouse.down();
  await page.waitForTimeout(160);
  console.log('  a 160 ms de maintien :', JSON.stringify(await etat()));
  await page.waitForTimeout(220);
  console.log('  a 380 ms de maintien :', JSON.stringify(await etat()));
  await page.mouse.up();
  await page.waitForTimeout(200);
  console.log('  relache sur place, ordre :', JSON.stringify(await ordre()));
  console.log('  et la tuile reposee :', JSON.stringify(await etat()));

  /**
   * Porter une tuile sur une autre, a la souris et pour de vrai.
   *
   * Les positions sont relues entre chaque pas : la grille se reorganise
   * pendant le geste, donc la case visee au depart n'est plus au meme
   * endroit une fois qu'elle a recule.
   */
  async function porter(depuis, vers) {
    const a = await centre(depuis);
    await page.mouse.move(a.x, a.y);
    await page.mouse.down();
    // Douze pixels, donc au dela du seuil : la tuile se souleve tout de
    // suite, sans attendre le quart de seconde.
    await page.mouse.move(a.x + 12, a.y, { steps: 3 });
    await page.waitForTimeout(120);
    const b = await centre(vers);
    await page.mouse.move(b.x, b.y, { steps: 12 });
    await page.waitForTimeout(220);
    const pendant = await etat();
    await page.mouse.up();
    await page.waitForTimeout(220);
    return pendant;
  }

  const pendant = await porter(0, 2);
  console.log('  pendant le geste :', JSON.stringify(pendant));
  console.log('  la premiere portee sur la troisieme :', JSON.stringify(await ordre()));
  console.log('  et la case creuse refermee :', JSON.stringify(await etat()));

  await porter(2, 0);
  console.log('  et ramenee sur la premiere :', JSON.stringify(await ordre()));

  // L'etiquette suit bien le curseur, et non un point fixe.
  const trois = await centre(2);
  await page.mouse.move(trois.x, trois.y);
  await page.mouse.down();
  await page.mouse.move(trois.x - 40, trois.y - 30, { steps: 6 });
  await page.waitForTimeout(200);
  const avant = await page.evaluate(() => {
    const e = document.querySelector('.porteur');
    return e ? { gauche: Math.round(parseFloat(e.style.left)), haut: Math.round(parseFloat(e.style.top)) } : null;
  });
  await page.mouse.move(trois.x - 200, trois.y - 30, { steps: 8 });
  await page.waitForTimeout(200);
  const apres = await page.evaluate(() => {
    const e = document.querySelector('.porteur');
    return e ? { gauche: Math.round(parseFloat(e.style.left)), haut: Math.round(parseFloat(e.style.top)) } : null;
  });
  console.log('  l etiquette portee, avant et apres :', JSON.stringify({ avant, apres }));
  await page.screenshot({ path: `${SORTIE}/tuiles-glisse.png` });
  await page.mouse.up();
  await page.waitForTimeout(220);

  // Le garde fou : un clic franc doit toujours ouvrir la tuile. Le
  // mecanisme de saisie s'installe sur le meme bouton de souris, donc
  // c'est exactement ce qu'un glisser mal borne casserait en premier.
  await page.evaluate(() => {
    const cases = [...document.querySelectorAll('.case')];
    const pop = cases.find((c) => c.dataset.tuile === 'popular');
    pop.scrollIntoView();
  });
  const pop = await page.evaluate(() => {
    const cases = [...document.querySelectorAll('.case')];
    const boite = cases
      .find((c) => c.dataset.tuile === 'popular')
      .querySelector('.tile')
      .getBoundingClientRect();
    return { x: boite.left + 40, y: boite.top + 40 };
  });
  await page.mouse.click(pop.x, pop.y);
  await page.waitForTimeout(300);
  console.log(
    '  un clic franc ouvre encore la tuile :',
    await page.evaluate(() => !!document.querySelector('.actions')),
  );
  await page.close();
}

/**
 * Ouvre l'ecran Editer.
 *
 * Il n'a plus d'onglet depuis l'etape 30 : le bandeau a cinq onglets de
 * la phase 4 ne lui en laisse pas, et il attend sa place definitive. En
 * attendant il se rejoint par Reglages, section Jeu, comme l'ancien
 * onglet des reglages du jeu avant lui.
 */
async function ouvrirEditer(page) {
  await page.locator('.head3-right button[aria-label="Launcher settings"]').click();
  await page.waitForTimeout(350);
  // On vise par la rangee et pas par le rang : le lot 23 a ajoute un
  // second bouton « Open » dans les Reglages, celui du journal, et ce
  // helper a commence a designer les deux a la fois.
  await page
    .locator('.bloc')
    .filter({ hasText: 'My game files' })
    .getByRole('button', { name: 'Open', exact: true })
    .click();
  await page.waitForTimeout(450);
}

// 11. L'ecran Editer
//
// Ce qu'on verifie n'est pas l'inventaire lui meme, qui est calcule et
// teste en Rust, mais ce que l'ecran en fait : les legendes, la fiche
// depliee, le bouton qui ouvre un dossier, celui qui ouvre un titre, et
// surtout le fait que le pack au nom bricole n'en ait PAS.
{
  const page = await capturer('editer', 1440, 900);
  await ouvrirEditer(page);

  const legendes = () =>
    page.evaluate(() =>
      [...document.querySelectorAll('.grille .tile')].map((t) => ({
        titre: t.querySelector('.libelle')?.textContent.trim(),
        legende: t.querySelector('.legende')?.textContent.trim(),
      })),
    );
  console.log('tuiles Editer :', JSON.stringify(await legendes()));

  // La fiche se deplie sur la tuile, une seule a la fois.
  await page.locator('[data-tuile="maps"] .tile').click();
  await page.waitForTimeout(300);
  console.log(
    '  fiche des maps :',
    JSON.stringify(
      await page.evaluate(() => {
        const fiche = document.querySelector('[data-tuile="maps"] .fiche');
        if (!fiche) return 'AUCUNE FICHE  <-- A CORRIGER';
        return {
          entetes: [...fiche.querySelectorAll('.entete')].map((e) => e.textContent.trim()),
          lignes: [...fiche.querySelectorAll('.ligne')].map((l) =>
            l.textContent.trim().replace(/\s+/g, ' '),
          ),
        };
      }),
    ),
  );

  await page.locator('[data-tuile="packs"] .tile').click();
  await page.waitForTimeout(300);
  console.log(
    '  une seule fiche ouverte a la fois :',
    await page.evaluate(() => document.querySelectorAll('.fiche').length),
  );

  // Le pack au nom bricole n'a pas d'identifiant de titre, donc pas de
  // bouton. C'est le controle qui compte : ce bouton part dans une ligne
  // de commande, et un fichier renomme a la main ne doit pas y donner
  // acces.
  console.log(
    '  boutons d ouverture dans la fiche des packs :',
    JSON.stringify(
      await page.evaluate(() => {
        // La colonne porte une classe, et pas un rang : la colonne des
        // emplacements disparait quand elle n'a rien a dire, et les
        // recents changent alors de place.
        const recents = document.querySelector('[data-tuile="packs"] .recents');
        return [...recents.querySelectorAll('.ligne')].map((l) => ({
          nom: l.querySelector('.nom')?.textContent.trim(),
          bouton: !!l.querySelector('.ouvrir'),
        }));
      }),
    ),
  );

  await page.locator('[data-tuile="packs"] .ouvrir').first().click();
  await page.waitForTimeout(250);
  console.log(
    '  titre demande au jeu :',
    await page.evaluate(() => window.__TITRE_OUVERT__ ?? 'AUCUN  <-- A CORRIGER'),
  );

  await page
    .locator('[data-tuile="packs"]')
    .getByRole('button', { name: 'Open folder', exact: true })
    .click();
  await page.waitForTimeout(250);
  console.log(
    '  dossier demande :',
    await page.evaluate(() => window.__DOSSIER_OUVERT__ ?? 'AUCUN  <-- A CORRIGER'),
  );
  await page.screenshot({ path: `${SORTIE}/editer.png` });

  // Les tuiles d'Editer se rangent comme celles de Jouer, par le meme
  // composant et la meme regle en Rust.
  const ordreEditer = () =>
    page.evaluate(() =>
      [...document.querySelectorAll('.grille .case')].map((c) => c.dataset.tuile),
    );
  console.log('  ordre au depart :', JSON.stringify(await ordreEditer()));
  const centre = (rang) =>
    page.evaluate((rang) => {
      const boite = document.querySelectorAll('.case')[rang].getBoundingClientRect();
      return { x: boite.left + boite.width / 2, y: boite.top + boite.height / 2 };
    }, rang);
  const a = await centre(0);
  await page.mouse.move(a.x, a.y);
  await page.mouse.down();
  await page.mouse.move(a.x + 12, a.y, { steps: 3 });
  await page.waitForTimeout(120);
  const b = await centre(2);
  await page.mouse.move(b.x, b.y, { steps: 12 });
  await page.waitForTimeout(220);
  await page.mouse.up();
  // L'ecriture des reglages est volontairement differee de 400 ms, il
  // faut donc la laisser partir avant de relire ce qui a ete garde.
  await page.waitForTimeout(700);
  console.log('  la premiere portee sur la troisieme :', JSON.stringify(await ordreEditer()));

  // Les deux ordres sont enregistres ensemble, et celui de Jouer n'a
  // pas bouge : c'est le controle qui compte, deux grilles qui partagent
  // un composant ne doivent pas partager leur ordre.
  console.log(
    '  ce qui est enregistre :',
    JSON.stringify(
      await page.evaluate(() => ({
        jouer: window.__GARDE__?.ordre_tuiles ?? 'rien enregistre',
        editer: window.__GARDE__?.ordre_tuiles_editer ?? 'rien enregistre',
      })),
    ),
  );

  // Un dossier vide doit le dire, pas afficher une fiche vide.
  await page.close();
  const vide = await capturer('editer-vide', 1440, 900, 'window.__SANS_FICHIERS__ = true;');
  await ouvrirEditer(vide);
  console.log(
    'sans aucun fichier :',
    JSON.stringify(
      await vide.evaluate(() =>
        [...document.querySelectorAll('.grille .legende')].map((l) => l.textContent.trim()),
      ),
    ),
  );
  await vide.close();
}

// 12. Le Classement et l'Historique
//
// Les donnees sont inventees, donc ce qui se verifie ici n'est pas leur
// exactitude mais la mise en page : les colonnes tiennent, la ligne du
// joueur se repere, le filtre filtre vraiment, et le pseudo le plus long
// du jeu d'essai ne pousse pas la table hors de la fenetre.
//
// Le controle qui compte est le dernier : **aucun debordement lateral**.
// Une table a six ou sept colonnes fixes est exactement ce qui casse a
// 1100 px, et la casse ne se voit pas sur une capture en 1440.
{
  const ouvrir = async (page, onglet) => {
    await page.locator(`.tab3-row button:has-text("${onglet}")`).first().click();
    await page.waitForTimeout(350);
  };

  const page = await capturer('classement', 1440, 900);
  await ouvrir(page, 'Leaderboard');
  await page.screenshot({ path: `${SORTIE}/classement.png` });

  console.log(
    'classement :',
    JSON.stringify(
      await page.evaluate(() => {
        const lignes = [...document.querySelectorAll('.liste .ligne:not(.entetes)')];
        const soi = document.querySelector('.ligne.soi');
        return {
          lignes: lignes.length,
          entetes: [...document.querySelectorAll('.entetes span')].map((e) =>
            e.textContent.trim(),
          ),
          // Une seule ligne marquee, et c'est bien celle du joueur.
          marquees: document.querySelectorAll('.ligne.soi').length,
          moi: soi?.querySelector('.pseudo')?.textContent.trim() ?? 'AUCUNE  <-- A CORRIGER',
          // La tete d'ecran porte le rang en grand et l'ecart a rattraper.
          place: document.querySelector('.moi .place')?.textContent.trim(),
          ecart: document.querySelector('.moi .ecart')?.textContent.trim().replace(/\s+/g, ' '),
          // La jauge du palier est remplie, et pas a zero par accident.
          jauge: document.querySelector('.moi .rempli')?.style.width ?? 'ABSENTE',
        };
      }),
    ),
  );

  // L'Historique, son filtre et son bilan.
  await ouvrir(page, 'History');
  await page.screenshot({ path: `${SORTIE}/historique.png` });

  const etat = () =>
    page.evaluate(() => ({
      lignes: document.querySelectorAll('.liste .ligne:not(.entetes)').length,
      // Le bilan doit suivre le filtre : c'est le cumul de ce qui est
      // AFFICHE, pas de tout l'historique.
      bilan: document.querySelector('.bilan')?.textContent.trim(),
      // Vise la table. La premiere version lisait `.pastille` sur toute
      // la page et ramassait la pastille d'etat du bandeau, donc une
      // chaine vide et un X qui ne venaient pas d'ici.
      marques: [
        ...new Set(
          [...document.querySelectorAll('.liste .marque')].map((p) => p.textContent.trim()),
        ),
      ],
    }));
  console.log('historique, tout :', JSON.stringify(await etat()));

  for (const choix of ['Wins', 'Losses']) {
    await page.locator(`.groupe button:has-text("${choix}")`).click();
    await page.waitForTimeout(250);
    console.log(`  filtre ${choix} :`, JSON.stringify(await etat()));
  }

  // Le signe est toujours ecrit, y compris le plus. Un oubli se verrait
  // ici : une colonne de deltas sans aucun signe.
  await page.locator('.groupe button:has-text("All")').click();
  await page.waitForTimeout(250);
  console.log(
    '  tous les deltas signes :',
    await page.evaluate(() =>
      [...document.querySelectorAll('.delta')].every((d) => /^[+-]/.test(d.textContent.trim())),
    ),
  );

  // La coherence des donnees d'exemple.
  //
  // Ce n'est pas du zele : les deux defauts controles ici ont tous les
  // deux ete presents, et aucun des deux ne se voit sur une capture. Les
  // dates remontaient le temps par endroits, et le total d'une equipe
  // pouvait etre inferieur a celui du joueur qui en fait partie. Une
  // donnee d'exemple incoherente fait douter de l'ecran.
  console.log(
    '  coherence des donnees :',
    JSON.stringify(
      await page.evaluate(() => {
        const lignes = [...document.querySelectorAll('.liste .ligne:not(.entetes)')];
        const chiffres = (l, rang) =>
          [...l.querySelectorAll('.stat')][rang].textContent.split('/').map((n) => +n.trim());
        // La date brute et non celle qui est affichee : celle-ci est
        // ecrite dans la langue lue, et un Date.parse qui echoue rend
        // NaN, que toutes les comparaisons declarent faux. Le controle
        // aurait alors annonce zero anomalie sur des dates illisibles.
        const valeurs = lignes.map((l) => +l.dataset.quand);
        return {
          datesIllisibles: valeurs.filter((v) => !Number.isFinite(v) || v === 0).length,
          datesQuiRemontent: valeurs.filter((v, i) => i > 0 && v >= valeurs[i - 1]).length,
          equipeSousLeJoueur: lignes.filter((l) => {
            const moi = chiffres(l, 0);
            const eq = chiffres(l, 1);
            return moi.some((v, i) => v > eq[i]);
          }).length,
        };
      }),
    ),
  );
  await page.close();

  // Le controle de debordement, aux trois largeurs du lot 17.
  for (const largeur of [1920, 1440, 1100]) {
    const p = await capturer(`historique-${largeur}`, largeur, 800);
    for (const [onglet, nom] of [['Leaderboard', 'classement'], ['History', 'historique']]) {
      await ouvrir(p, onglet);
      const debord = await p.evaluate(() => {
        const ecran = document.querySelector('.ecran');
        const lignes = [...document.querySelectorAll('.ligne')];
        const cadre = ecran.getBoundingClientRect();
        const trop = lignes.filter((l) => l.scrollWidth > l.clientWidth + 1).length;
        return {
          debordeALaPage: document.documentElement.scrollWidth > window.innerWidth,
          colonnesCoupees: trop,
          largeurLigne: lignes[1] ? +lignes[1].getBoundingClientRect().width.toFixed(1) : null,
          cadre: +cadre.width.toFixed(1),
        };
      });
      console.log(`  ${nom} a ${largeur} :`, JSON.stringify(debord));
    }
    await p.screenshot({ path: `${SORTIE}/historique-${largeur}.png` });
    await p.close();
  }

  // Et en francais, la ou les libelles sont les plus longs.
  const fr = await capturer('classement-fr', 1440, 900, "window.__REGLAGES__ = { langue: 'fr' };");
  await ouvrir(fr, 'Classement');
  await fr.screenshot({ path: `${SORTIE}/classement-fr.png` });
  await ouvrir(fr, 'Historique');
  await fr.screenshot({ path: `${SORTIE}/historique-fr.png` });
  console.log(
    'en francais :',
    JSON.stringify(
      await fr.evaluate(() => ({
        entetes: [...document.querySelectorAll('.entetes span')].map((e) => e.textContent.trim()),
        filtres: [...document.querySelectorAll('.groupe button')].map((b) => b.textContent.trim()),
        marques: [
          ...new Set(
            [...document.querySelectorAll('.liste .marque')].map((m) => m.textContent.trim()),
          ),
        ],
      })),
    ),
  );
  await fr.close();
}

// 13. L'ecran Developpeur, le lecteur de journal
//
// Ce qui se verifie ici n'est PAS l'anonymisation ni le decoupage des
// lignes : ils vivent en Rust et y ont trente-deux tests. Ce qui se
// verifie est ce que l'interface DEMANDE et ce qu'elle fait de la
// reponse, c'est a dire exactement la part qu'aucun test Rust ne couvre.
//
// Le controle qui compte est le dernier : **le rapport ne prend aucun
// argument**. Si l'interface pouvait lui passer le choix d'affichage,
// quelqu'un qui a coche « chemins complets » enverrait un jour son
// dossier utilisateur dans un salon de discussion. Ce n'est pas une
// question de discipline, c'est une question de forme d'appel.
{
  // Un echantillon qui melange les trois niveaux, et qui DEPASSE la
  // limite d'affichage : sans ca la troncature ne se declenche pas et le
  // controle qui la vise passerait au vert sans avoir rien vu.
  const lignes = [];
  for (let i = 1; i <= 1150; i++) {
    const niveau = i % 10 === 0 ? 'erreur' : i % 4 === 0 ? 'alerte' : 'info';
    lignes.push({
      numero: i,
      horodatage: `2026-09-30 12:00:${String(i % 60).padStart(2, '0')}.000`,
      niveau,
      message: `${niveau === 'erreur' ? 'ManiaPlanet.exe introuvable' : 'ligne ordinaire'} numero ${i}`,
    });
  }
  const avant = `window.__LIGNES_JOURNAL__ = ${JSON.stringify(lignes)};`;

  const page = await capturer('developpeur', 1440, 900, avant);
  await page.locator('.head3-right button[aria-label="Launcher settings"]').click();
  await page.waitForTimeout(350);
  // Deux boutons « Open » dans les Reglages depuis l'etape 30, celui de
  // l'ecran Editer et celui-ci. On vise par la rangee et pas par le rang.
  await page
    .locator('.bloc')
    .filter({ hasText: 'Log and problem report' })
    .getByRole('button', { name: 'Open', exact: true })
    .click();
  await page.waitForTimeout(600);
  await page.screenshot({ path: `${SORTIE}/developpeur.png` });

  const dernierAppel = () =>
    page.evaluate(() => (window.__APPELS_JOURNAL__ || []).slice(-1)[0] ?? null);
  const affichees = () =>
    page.evaluate(() => ({
      lignes: document.querySelectorAll('.liste .ligne').length,
      comptes: [...document.querySelectorAll('.niveau')].map((b) =>
        b.textContent.trim().replace(/\s+/g, ' '),
      ),
      tronque: document.querySelector('.tronque')?.textContent.trim().replace(/\s+/g, ' ') ?? null,
    }));

  // L'alignement sur la gouttiere, mesure et pas regarde.
  //
  // Plafonner la largeur d'un `.pad` le detache de la gouttiere, parce
  // qu'il se centre par marges automatiques. L'etape 32 s'est fait avoir
  // sur la table de l'Historique, le lot 25 s'est fait avoir sur la
  // liste des articles. Deux fois la meme erreur veut dire qu'un
  // commentaire ne suffit pas : ce controle couvre les trois ecrans
  // concernes, a la largeur ou le plafond mord.
  for (const largeur of [1920, 1440]) {
    const p = await capturer(`alignement-${largeur}`, largeur, 800, avant);
    const mesures = [];
    for (const [onglet, titre, liste] of [
      ['Community', '.titre', '.carte'],
      ['History', '.titre', '.liste .ligne'],
      ['Leaderboard', '.titre', '.liste .ligne'],
    ]) {
      await p.locator(`.tab3-row button:has-text("${onglet}")`).first().click();
      await p.waitForTimeout(450);
      mesures.push(
        await p.evaluate(
          ([t, l, nom]) => {
            const gauche = (s) => {
              const e = document.querySelector(s);
              return e ? +e.getBoundingClientRect().left.toFixed(1) : null;
            };
            return { ecran: nom, titre: gauche(t), liste: gauche(l) };
          },
          [titre, liste, onglet],
        ),
      );
    }
    const decales = mesures.filter(
      (m) => m.titre !== null && m.liste !== null && Math.abs(m.titre - m.liste) > 1,
    );
    console.log(
      `  alignement a ${largeur} :`,
      decales.length === 0 ? 'les trois sur la gouttiere' : JSON.stringify(decales),
    );
    await p.close();
  }

  console.log('a l ouverture :', JSON.stringify(await dernierAppel()));
  console.log('  affiche :', JSON.stringify(await affichees()));

  // Le filtre. Les comptes doivent rester ceux du FICHIER : c'est le
  // piege de ce genre d'ecran, un compte qui tombe a zero des qu'on
  // decoche ne dit plus rien de ce que contient le journal.
  await page.locator('.niveau', { hasText: 'Info' }).click();
  await page.waitForTimeout(350);
  console.log('  sans les infos, demande :', JSON.stringify(await dernierAppel()));
  console.log('  sans les infos, affiche :', JSON.stringify(await affichees()));

  await page.locator('.niveau', { hasText: 'Warnings' }).click();
  await page.locator('.niveau', { hasText: 'Errors' }).click();
  await page.waitForTimeout(350);
  console.log('  aucun niveau coche :', JSON.stringify(await affichees()));

  // On recoche tout, puis la recherche.
  await page.locator('.niveau', { hasText: 'Info' }).click();
  await page.locator('.niveau', { hasText: 'Warnings' }).click();
  await page.locator('.niveau', { hasText: 'Errors' }).click();
  await page.waitForTimeout(350);
  await page.locator('.champ').fill('introuvable');
  await page.waitForTimeout(400);
  console.log('  recherche, demande :', JSON.stringify(await dernierAppel()));
  console.log('  recherche, affiche :', JSON.stringify(await affichees()));
  await page.locator('.champ').fill('');
  await page.waitForTimeout(400);

  // La troncature, et le bouton qui la leve.
  console.log('  troncature :', JSON.stringify(await affichees()));
  await page.locator('.tronque button').click();
  await page.waitForTimeout(400);
  console.log('  apres Tout afficher, demande :', JSON.stringify(await dernierAppel()));
  console.log('  apres Tout afficher, affiche :', JSON.stringify(await affichees()));

  // Le suivi. Il commande aussi le rechargement, donc remonter dans la
  // liste doit l'arreter, et le bouton doit le rendre.
  const etatSuivi = () =>
    page.evaluate(() => ({
      enDirect: !!document.querySelector('.direct'),
      boutonReprendre: !!document.querySelector('.reprendre'),
      // Colle-t-on vraiment au bas de la liste ?
      colleEnBas: (() => {
        const l = document.querySelector('.liste');
        return l ? l.scrollHeight - l.scrollTop - l.clientHeight < 24 : null;
      })(),
    }));
  console.log('  suivi a l ouverture :', JSON.stringify(await etatSuivi()));
  await page.evaluate(() => {
    document.querySelector('.liste').scrollTop = 0;
    document.querySelector('.liste').dispatchEvent(new Event('scroll'));
  });
  await page.waitForTimeout(250);
  console.log('  apres etre remonte :', JSON.stringify(await etatSuivi()));

  // Et le rechargement s'est bien tu : on laisse passer plus d'une
  // periode et le nombre d'appels ne doit pas bouger.
  const appelsAvant = await page.evaluate(() => window.__APPELS_JOURNAL__.length);
  await page.waitForTimeout(2600);
  const appelsApres = await page.evaluate(() => window.__APPELS_JOURNAL__.length);
  console.log(`  appels pendant la pause : ${appelsApres - appelsAvant}`);

  await page.locator('.reprendre').click();
  await page.waitForTimeout(2600);
  console.log('  apres Reprendre le suivi :', JSON.stringify(await etatSuivi()));
  console.log(
    `  appels pendant le suivi : ${(await page.evaluate(() => window.__APPELS_JOURNAL__.length)) - appelsApres}`,
  );

  // L'interrupteur des chemins complets doit vraiment changer ce qui est
  // demande. Un interrupteur qui ne fait rien est pire qu'un interrupteur
  // absent, et il ne se voit pas sur une capture.
  await page.locator('.bascule button').click();
  await page.waitForTimeout(400);
  console.log('  chemins complets, demande :', JSON.stringify(await dernierAppel()));
  await page.locator('.bascule button').click();
  await page.waitForTimeout(400);
  console.log('  chemins masques, demande :', JSON.stringify(await dernierAppel()));

  // Le rapport. Le controle structurel : aucun argument.
  await page.getByRole('button', { name: 'Copy report', exact: true }).click();
  await page.waitForTimeout(400);
  console.log(
    '  rapport :',
    JSON.stringify(
      await page.evaluate(() => ({
        demandes: window.__RAPPORTS_DEMANDES__ ?? 0,
        arguments: window.__ARGS_RAPPORT__ ?? null,
        note: document.querySelector('.note')?.textContent.trim() ?? 'AUCUNE  <-- A CORRIGER',
      })),
    ),
  );

  await page.getByRole('button', { name: 'Save report', exact: true }).click();
  await page.waitForTimeout(400);
  console.log(
    '  enregistrement :',
    JSON.stringify(
      await page.evaluate(() => ({
        ecrit: !!window.__RAPPORT_ECRIT__,
        note: document.querySelector('.note')?.textContent.trim() ?? 'AUCUNE  <-- A CORRIGER',
      })),
    ),
  );

  await page
    .locator('.rapport')
    .getByRole('button', { name: 'Open folder', exact: true })
    .click();
  await page.waitForTimeout(250);
  console.log(
    '  dossier du journal :',
    await page.evaluate(() => window.__DOSSIER_JOURNAL_OUVERT__ === true),
  );
  await page.close();

  // Un journal vide doit le dire, et ne pas montrer un cadre vide.
  const vide = await capturer('developpeur-vide', 1440, 900, 'window.__LIGNES_JOURNAL__ = [];');
  await vide.locator('.head3-right button[aria-label="Launcher settings"]').click();
  await vide.waitForTimeout(350);
  await vide
    .locator('.bloc')
    .filter({ hasText: 'Log and problem report' })
    .getByRole('button', { name: 'Open', exact: true })
    .click();
  await vide.waitForTimeout(500);
  console.log(
    'journal vide :',
    await vide.evaluate(() => document.querySelector('.vide')?.textContent.trim() ?? 'RIEN'),
  );
  await vide.screenshot({ path: `${SORTIE}/developpeur-vide.png` });
  await vide.close();

  // Et en francais.
  const fr = await capturer(
    'developpeur-fr',
    1440,
    900,
    `window.__REGLAGES__ = { langue: 'fr' }; ${avant}`,
  );
  await fr.locator('.head3-right button[aria-label="Réglages du launcher"]').click();
  await fr.waitForTimeout(350);
  await fr
    .locator('.bloc')
    .filter({ hasText: 'Journal et rapport' })
    .getByRole('button', { name: 'Ouvrir', exact: true })
    .click();
  await fr.waitForTimeout(500);
  await fr.screenshot({ path: `${SORTIE}/developpeur-fr.png` });
  console.log(
    'en francais :',
    JSON.stringify(
      await fr.evaluate(() => ({
        titre: document.querySelector('.titre')?.textContent.trim(),
        niveaux: [...document.querySelectorAll('.niveau')].map((b) =>
          b.textContent.trim().replace(/\s+/g, ' '),
        ),
        pied: document.querySelector('.explication')?.textContent.trim(),
      })),
    ),
  );
  await fr.close();
}

// 14. L'ecran Communaute, le fil d'actualites
//
// La signature et le decoupage du markdown sont en Rust et y sont
// testes. Ce qui se verifie ici est la part qu'aucun test Rust ne
// couvre : ce que l'interface demande, et ce qu'elle DESSINE.
//
// Le controle qui compte est le dernier : **un article qui contient une
// balise reste du texte**. Il n'y a pas d'echappement a faire parce
// qu'il n'y a pas de HTML, et c'est precisement ce qu'on veut prouver
// plutot que croire.
{
  const articles = [
    {
      id: 'a', titre: 'Coupe Atria, samedi 21h', categorie: 'tournoi',
      date: 1791013600, resume: 'Inscriptions jusqu a vendredi.', langue: 'fr', epingle: false,
      corps: [{ sorte: 'paragraphe', fragments: [{ sorte: 'texte', texte: 'En francais.' }] }],
    },
    {
      // Sans langue, donc visible dans les deux. C'est celui qu'on
      // deplie, puisque la page du banc est en anglais par defaut et que
      // le filtre de langue ecarte les articles francais, ce qu'il doit
      // faire.
      id: 'b', titre: 'Frostline', categorie: 'maps',
      date: 1790927200, resume: 'Huit maps Elite.', langue: '', epingle: false,
      corps: [
        { sorte: 'paragraphe', fragments: [
          { sorte: 'texte', texte: 'La coupe reprend ' },
          { sorte: 'gras', texte: 'samedi' },
          { sorte: 'texte', texte: ', en Elite. Voir ' },
          { sorte: 'lien', texte: 'le reglement', lien: 'https://exemple.org/reglement' },
          { sorte: 'texte', texte: '.' },
        ] },
        { sorte: 'liste', ordonnee: true, points: [
          [{ sorte: 'texte', texte: 'Inscris ton equipe' }],
          [{ sorte: 'texte', texte: 'Presente toi a 20h45' }],
        ] },
        { sorte: 'citation', fragments: [{ sorte: 'texte', texte: 'Une citation.' }] },
        { sorte: 'code', texte: '# pas un titre' },
      ],
    },
    {
      // Le controle d'injection : ces deux fragments doivent s'afficher
      // tels quels, comme du texte. Sans langue, pour etre visible.
      id: 'inj', titre: 'Texte qui ressemble a du code', categorie: 'technique',
      date: 1790840000, resume: 'Doit rester du texte.', langue: '', epingle: false,
      corps: [{ sorte: 'paragraphe', fragments: [
        { sorte: 'texte', texte: '<script>window.__INJECTE__ = true;<\/script>' },
        { sorte: 'texte', texte: '<img src=x onerror="window.__INJECTE__=true">' },
      ] }],
    },
    {
      id: 'c', titre: 'Bienvenue', categorie: 'technique',
      date: 1791100000, resume: 'Le fil ouvre.', langue: 'fr', epingle: true,
      corps: [{ sorte: 'titre', niveau: 1, texte: 'Un titre' }],
    },
    {
      id: 'd', titre: 'English only', categorie: 'maps',
      date: 1791100001, resume: '', langue: 'en', epingle: false, corps: [],
    },
  ];
  const avant = `window.__ARTICLES__ = ${JSON.stringify(articles)};`;

  const page = await capturer('communaute', 1440, 900, avant);
  await page.locator('.tab3-row button:has-text("Community")').first().click();
  await page.waitForTimeout(600);
  await page.screenshot({ path: `${SORTIE}/communaute.png` });

  const dernierAppel = () =>
    page.evaluate(() => (window.__APPELS_ACTUS__ || []).slice(-1)[0] ?? null);
  const affiche = () =>
    page.evaluate(() => ({
      cartes: [...document.querySelectorAll('.carte .nom')].map((n) => n.textContent.trim()),
      categories: [...document.querySelectorAll('.categorie')].map((b) =>
        b.textContent.trim().replace(/\s+/g, ' '),
      ),
      avertissement: document.querySelector('.avertissement')?.textContent.trim() ?? null,
    }));

  console.log('a l ouverture :', JSON.stringify(await dernierAppel()));
  console.log('  affiche :', JSON.stringify(await affiche()));
  // La page du banc est en anglais : les articles marques `fr` ne
  // doivent pas s'y trouver, et ceux sans langue doivent y etre.
  console.log(
    '  filtre de langue :',
    JSON.stringify(
      await page.evaluate(() => {
        const titres = [...document.querySelectorAll('.carte .nom')].map((n) =>
          n.textContent.trim(),
        );
        return {
          francaisAffiches: titres.filter((t) => t.includes('Coupe Atria')).length,
          sansLangueAffiches: titres.filter((t) => t === 'Frostline').length,
        };
      }),
    ),
  );

  // L'anglais ne doit pas apparaitre en francais, et l'article sans
  // langue doit apparaitre dans les deux.
  await page.locator('.categorie', { hasText: 'Maps' }).click();
  await page.waitForTimeout(350);
  console.log('  filtre Maps, demande :', JSON.stringify(await dernierAppel()));
  console.log('  filtre Maps, affiche :', JSON.stringify(await affiche()));
  await page.locator('.categorie', { hasText: 'Maps' }).click();
  await page.waitForTimeout(300);

  await page.locator('.champ').fill('frostline');
  await page.waitForTimeout(400);
  console.log('  recherche :', JSON.stringify(await affiche()));
  await page.locator('.champ').fill('');
  await page.waitForTimeout(400);

  // Le depliage, une carte a la fois, et le corps dessine.
  await page.locator('.carte').filter({ hasText: 'Frostline' }).locator('.tete').click();
  await page.waitForTimeout(300);
  console.log(
    '  article deplie :',
    JSON.stringify(
      await page.evaluate(() => {
        const c = document.querySelector('.carte.deplie .contenu');
        return {
          deplies: document.querySelectorAll('.carte.deplie').length,
          gras: !!c?.querySelector('strong'),
          listeOrdonnee: !!c?.querySelector('ol'),
          citation: !!c?.querySelector('blockquote'),
          code: c?.querySelector('pre')?.textContent ?? null,
          liens: [...(c?.querySelectorAll('.lien') ?? [])].map((l) => l.textContent.trim()),
        };
      }),
    ),
  );

  // Un lien part au systeme et ne navigue pas dans la fenetre.
  const avantUrl = page.url();
  await page.locator('.carte.deplie .lien').first().click();
  await page.waitForTimeout(300);
  console.log(
    '  clic sur un lien :',
    JSON.stringify(
      await page.evaluate(() => ({ ouverts: window.__LIENS_OUVERTS__ ?? [] })),
    ),
    `| la fenetre a bouge : ${page.url() !== avantUrl}`,
  );

  // LE CONTROLE QUI COMPTE. Un article porte deux fragments qui, s'ils
  // passaient par du HTML, executeraient du code. Ils doivent etre du
  // texte, et rien ne doit s'etre execute.
  await page.locator('.carte').filter({ hasText: 'ressemble' }).locator('.tete').click();
  await page.waitForTimeout(350);
  console.log(
    '  injection :',
    JSON.stringify(
      await page.evaluate(() => {
        const c = document.querySelector('.carte.deplie .contenu');
        return {
          codeExecute: window.__INJECTE__ === true,
          balisesCreees: (c?.querySelectorAll('script, img') ?? []).length,
          texteAffiche: c?.textContent.trim().slice(0, 60),
        };
      }),
    ),
  );
  await page.close();

  // L'accueil porte maintenant de vrais articles, donc le banc regarde
  // aussi la place la plus visible de l'application.
  const accueil = await capturer('accueil-actualites', 1440, 900, avant);
  await accueil.waitForTimeout(500);
  console.log(
    'accueil, Quoi de neuf :',
    JSON.stringify(
      await accueil.evaluate(() => ({
        cartes: [...document.querySelectorAll('.pile .carte-titre')].map((t) =>
          t.textContent.trim(),
        ),
        vues: document.querySelectorAll('.pile .paire').length,
        points: document.querySelectorAll('.points .dot').length,
      })),
    ),
  );
  await accueil.screenshot({ path: `${SORTIE}/accueil-actualites.png` });
  // Une carte mene au fil.
  await accueil.locator('.pile .paire:not(.cachee) .carte.vers').first().click();
  await accueil.waitForTimeout(400);
  console.log(
    '  une carte mene a :',
    await accueil.evaluate(() => (document.querySelector('.carte .nom') ? 'community' : 'ailleurs')),
  );
  await accueil.close();

  // Sans actualites, l'accueil le dit au lieu de montrer une rangee vide.
  const accueilVide = await capturer('accueil-sans-actus', 1440, 900, 'window.__ARTICLES__ = [];');
  await accueilVide.waitForTimeout(500);
  console.log(
    'accueil sans actualites :',
    JSON.stringify(
      await accueilVide.evaluate(() => ({
        cartes: document.querySelectorAll('.pile .carte').length,
        texte: document.querySelector('.pile .corps')?.textContent.trim() ?? 'RIEN  <-- A CORRIGER',
        points: document.querySelectorAll('.points .dot').length,
      })),
    ),
  );
  await accueilVide.close();

  // Reseau coupe mais cache valable : des articles ET un avertissement.
  const horsLigne = await capturer(
    'communaute-hors-ligne', 1440, 900,
    `window.__ACTUS_HORS_LIGNE__ = true; ${avant}`,
  );
  await horsLigne.locator('.tab3-row button:has-text("Community")').first().click();
  await horsLigne.waitForTimeout(600);
  console.log(
    'hors ligne avec cache :',
    JSON.stringify(
      await horsLigne.evaluate(() => ({
        cartes: document.querySelectorAll('.carte').length,
        avertissement: document.querySelector('.avertissement')?.textContent.trim() ?? 'AUCUN  <-- A CORRIGER',
      })),
    ),
  );
  await horsLigne.screenshot({ path: `${SORTIE}/communaute-hors-ligne.png` });
  await horsLigne.close();

  // Rien du tout : on le dit, on ne laisse pas une page blanche.
  const panne = await capturer('communaute-vide', 1440, 900, 'window.__ACTUS_EN_PANNE__ = true;');
  await panne.locator('.tab3-row button:has-text("Community")').first().click();
  await panne.waitForTimeout(600);
  console.log(
    'aucune actualite :',
    await panne.evaluate(
      () => document.querySelector('.avertissement, .vide')?.textContent.trim() ?? 'RIEN  <-- A CORRIGER',
    ),
  );
  await panne.screenshot({ path: `${SORTIE}/communaute-vide.png` });
  await panne.close();

  // Pas encore configure. C'est l'etat de depart, tant qu'aucun fil n'a
  // ete publie, et il ne doit pas se lire comme une panne de reseau.
  const neuf = await capturer(
    'communaute-non-configure', 1440, 900,
    'window.__ACTUS_EN_PANNE__ = true; window.__ACTUS_NON_CONFIGUREES__ = true;',
  );
  await neuf.locator('.tab3-row button:has-text("Community")').first().click();
  await neuf.waitForTimeout(600);
  console.log(
    'pas encore configure :',
    await neuf.evaluate(
      () => document.querySelector('.avertissement, .vide')?.textContent.trim() ?? 'RIEN  <-- A CORRIGER',
    ),
  );
  await neuf.screenshot({ path: `${SORTIE}/communaute-non-configure.png` });
  await neuf.close();

  // Et en francais.
  const fr = await capturer(
    'communaute-fr', 1440, 900,
    `window.__REGLAGES__ = { langue: 'fr' }; ${avant}`,
  );
  await fr.locator('.tab3-row button:has-text("Communauté")').first().click();
  await fr.waitForTimeout(600);
  await fr.screenshot({ path: `${SORTIE}/communaute-fr.png` });
  console.log(
    'en francais :',
    JSON.stringify(
      await fr.evaluate(() => ({
        langueDemandee: (window.__APPELS_ACTUS__ || []).slice(-1)[0]?.langue,
        cartes: [...document.querySelectorAll('.carte .nom')].map((n) => n.textContent.trim()),
        categories: [...document.querySelectorAll('.categorie')].map((b) =>
          b.textContent.trim().replace(/\s+/g, ' '),
        ),
      })),
    ),
  );
  await fr.close();
}

console.log('erreurs console:', erreurs.length ? erreurs : 'aucune');
await navigateur.close();
serveur.close();
