// Rend la maquette d'origine dans les memes conditions que le portage,
// pour comparer ce qui doit l'etre : la planche, le voile, et le texte
// par dessus.
import { chromium } from 'playwright';
import { pathToFileURL } from 'node:url';

// Le chemin de la maquette, en premier argument. Par defaut, la ou elle
// vit chez Flow, a cote du dossier du projet.
const MAQUETTE =
  process.argv[2] ??
  new URL('../../ShootMania Launcher POC/shootmania-launcher-demo.html', import.meta.url)
    .pathname;

// executablePath n'est utile que dans un conteneur ou Playwright ne
// gere pas lui meme ses navigateurs. Sur un poste ordinaire, laisser
// vide : npx playwright install chromium suffit.
const navigateur = await chromium.launch(
  process.env.CHROMIUM ? { executablePath: process.env.CHROMIUM } : {},
);
const page = await navigateur.newPage({ viewport: { width: 1440, height: 900 } });
await page.goto(pathToFileURL(MAQUETTE).href);
await page.waitForTimeout(600);

// La maquette s'ouvre sur l'assistant. On le passe pour arriver a l'ecran
// Jouer, qui est ce qu'on veut comparer.
for (let i = 0; i < 3; i++) {
  const bouton = page.locator('[data-a="next"], [data-a="finish"]').first();
  if (await bouton.count()) {
    await bouton.click();
    await page.waitForTimeout(350);
  }
}
await page.waitForTimeout(400);
await page.screenshot({ path: '/tmp/extrait/maquette-1440.png' });

const mesures = await page.evaluate(() => {
  const tuile = document.querySelector('.tile');
  const vec = document.querySelector('.tile-vec');
  const svg = document.querySelector('.tile-svg');
  const nav = document.querySelector('.tab3-row');
  return {
    opaciteVec: vec ? getComputedStyle(vec).opacity : null,
    tuileHauteur: tuile ? +tuile.getBoundingClientRect().height.toFixed(1) : null,
    tuileLargeur: tuile ? +tuile.getBoundingClientRect().width.toFixed(1) : null,
    svgBoite: svg
      ? {
          l: +svg.getBoundingClientRect().width.toFixed(1),
          h: +svg.getBoundingClientRect().height.toFixed(1),
        }
      : null,
    premiereTuile: tuile ? +tuile.getBoundingClientRect().top.toFixed(1) : null,
    hauteurBandeau: nav
      ? +document.querySelector('.tab3-row').closest('div').parentElement.getBoundingClientRect()
          .height.toFixed(1)
      : null,
    legendePopular: document.querySelectorAll('.tile-body div')[3]?.textContent,
  };
});
console.log('maquette:', JSON.stringify(mesures, null, 1));
await navigateur.close();
