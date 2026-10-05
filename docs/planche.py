# -*- coding: utf-8 -*-
"""Fabrique la planche d'essai des emblemes de palier.

Jetable. Elle sert a trancher une direction, pas a etre maintenue.
"""

PALIERS = [
    ("Bronze", "#C08457"),
    ("Argent", "#AEB9C4"),
    ("Or", "#E8B93E"),
    ("Platine", "#5FD4D0"),
    ("Diamant", "#9B8CFF"),
]

# ---------------------------------------------------------------- A, le cairn
# Des pierres empilees. Trois corrections apres le premier jet, ou elles
# se collaient en une seule montagne :
#   1. un vrai intervalle entre chaque pierre, et pas un simple contour,
#      parce qu'un contour de 0,8 disparait a 18 px ;
#   2. des largeurs ET des decalages irreguliers, pour que la silhouette
#      soit crantee. Une pile centree et reguliere fait une pyramide,
#      et une pyramide ne se compte pas ;
#   3. plus de ligne de sol : elle se lisait comme une ombre.

def pierre(x, w, y, h, r=1.0, angle=0.0):
    rect = (f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" '
            'fill="currentColor"/>')
    if angle == 0.0:
        return rect
    cx, cy = x + w / 2, y + h / 2
    return f'<g transform="rotate({angle} {cx:.2f} {cy:.2f})">{rect}</g>'

P0 = pierre(3.6, 16.8, 17.2, 4.0, 1.2, -2.4)
P1 = pierre(6.2, 11.6, 12.5, 3.6, 1.0, 3.2)
P2 = pierre(8.8, 10.2, 8.0, 3.4, 0.9, -3.4)

CLE = ('<g transform="rotate(2.6 12 4.6)">'
       '<path d="M8.2,7.2 L12.0,2.0 L15.8,7.2 Z" fill="currentColor" '
       'stroke="currentColor" stroke-width="1.1" stroke-linejoin="round"/></g>')

CRISTAL = (
    '<g transform="rotate(4 12.3 4.4)">'
    '<path d="M12.3,1.0 L16.3,4.4 L12.3,7.8 L8.3,4.4 Z" fill="currentColor" '
    'stroke="currentColor" stroke-width="1.1" stroke-linejoin="round"/>'
    '<path d="M12.3,1.0 L12.3,7.8 L8.3,4.4 Z" fill="#fff" opacity="0.32"/>'
    '</g>'
)

A = [
    P0,
    P0 + P1,
    P0 + P1 + P2,
    P0 + P1 + P2 + CLE,
    P0 + P1 + P2 + CRISTAL,
]

# ------------------------------------------------------------- B, la marque
# L'hexagone et le cube isometrique du logo, qui se remplissent.
#
# Le logo lui meme porte une sphere lumineuse au centre du cube. Aucun
# palier ne la reprend, donc aucun palier n'est exactement la marque.
#
# Diamant ne se distingue plus par six clous, invisibles a 18 px, mais
# par un second hexagone autour : ca change la SILHOUETTE, et la
# silhouette est la seule chose qui survive a la reduction.

HEX = 'M12,2 L20.66,7 L20.66,17 L12,22 L3.34,17 L3.34,7 Z'
HEX_HALO = 'M12,0.2 L21.9,5.9 L21.9,18.1 L12,23.8 L2.1,18.1 L2.1,5.9 Z'
FACE_G = 'M3.34,7 L12,12 L12,22 L3.34,17 Z'
FACE_D = 'M20.66,7 L12,12 L12,22 L20.66,17 Z'
FACE_H = 'M12,2 L20.66,7 L12,12 L3.34,7 Z'


def hexa(faces, halo=False):
    out = []
    if halo:
        out.append(f'<path d="{HEX_HALO}" fill="currentColor" opacity="0.3"/>')
        out.append(f'<path d="{HEX_HALO}" fill="none" stroke="currentColor" '
                   'stroke-width="1.5" stroke-linejoin="round" opacity="0.85"/>')
    out.append(f'<path d="{HEX}" fill="none" stroke="currentColor" '
               'stroke-width="1.7" stroke-linejoin="round"/>')
    for f, op in faces:
        out.append(f'<path d="{f}" fill="currentColor" opacity="{op}"/>')
    return ''.join(out)


CUBE = [(FACE_G, 0.92), (FACE_D, 0.55), (FACE_H, 1)]

B = [
    hexa([]),
    hexa([(FACE_G, 0.92)]),
    hexa([(FACE_G, 0.92), (FACE_D, 0.55)]),
    hexa(CUBE),
    hexa(CUBE, halo=True),
]

# ----------------------------------------------------------- C, les chevrons
# Des accents empiles, le dispositif de grade le plus ancien qui soit.
#
# Diamant ne gagne plus un losange pose au dessus, qui se collait au
# chevron du dessous : son accent du haut devient PLEIN. Un plein contre
# un trait se distingue a n'importe quelle taille.

def chev(y):
    return (f'<path d="M5,{y:.1f} L12,{y-4.4:.1f} L19,{y:.1f}" fill="none" '
            'stroke="currentColor" stroke-width="2.3" stroke-linecap="round" '
            'stroke-linejoin="round"/>')


def chev_plein(y):
    return (f'<path d="M5,{y:.1f} L12,{y-6.2:.1f} L19,{y:.1f} Z" '
            'fill="currentColor" stroke="currentColor" stroke-width="1.4" '
            'stroke-linejoin="round"/>')


BARRE = ('<path d="M5.4,21.5 L18.6,21.5" fill="none" stroke="currentColor" '
         'stroke-width="2.3" stroke-linecap="round"/>')

C = [
    chev(15.2),
    chev(18.0) + chev(13.0),
    chev(19.4) + chev(14.4) + chev(9.4),
    BARRE + chev(18.2) + chev(13.6) + chev(9.0),
    BARRE + chev(18.2) + chev(13.6) + chev_plein(9.0),
]

DIRECTIONS = [
    ("A", "Le cairn", "Des pierres empilees, de largeurs et de decalages irreguliers pour que la silhouette soit crantee. Bronze une pierre, puis deux, trois, la cle de voute, et la meme cle devenue cristal. C'est le nom du projet qui parle.", A),
    ("B", "La marque", "L'hexagone du logo qui se remplit face par face jusqu'au cube complet, puis le cube cercle d'un second hexagone. Unite totale avec l'identite. La marque, elle, porte une sphere au centre du cube qu'aucun palier ne reprend : aucun rang n'est donc exactement le logo.", B),
    ("C", "Les chevrons", "Des accents empiles, le dispositif de grade le plus ancien qui soit, plus vieux que n'importe quel jeu. Un, deux, trois, la barre, et l'accent du haut qui devient plein.", C),
]


def svg(trace, taille, couleur):
    return (f'<svg width="{taille}" height="{taille}" viewBox="0 0 24 24" '
            f'style="color:{couleur}" aria-hidden="true">{trace}</svg>')


morceaux = []
for lettre, nom, note, traces in DIRECTIONS:
    lignes = []
    # Les trois tailles, en couleur.
    for taille, legende in [(160, "160 px, la taille du profil"),
                            (40, "40 px, la carte de tete du classement"),
                            (18, "18 px, la colonne du classement")]:
        cases = ''.join(
            f'<div class="case"><div class="boite" style="height:{max(taille,22)}px">'
            f'{svg(t, taille, c)}</div>'
            + (f'<span class="nom">{p}</span>' if taille == 160 else '')
            + '</div>'
            for t, (p, c) in zip(traces, PALIERS)
        )
        lignes.append(f'<div class="rangee"><p class="legende">{legende}</p>'
                      f'<div class="cases">{cases}</div></div>')
    # Le test sans couleur.
    cases = ''.join(
        f'<div class="case"><div class="boite" style="height:22px">'
        f'{svg(t, 18, "#C9D1D9")}</div></div>' for t in traces
    )
    lignes.append('<div class="rangee epreuve"><p class="legende">'
                  '18 px, sans la couleur. <b>C\'est le test qui compte.</b> '
                  'Si les cinq ne se distinguent pas ici, la forme ne fait '
                  'pas son travail et c\'est la couleur qui porte tout.'
                  f'</p><div class="cases">{cases}</div></div>')
    # En situation, dans une ligne de classement.
    rangs = ''.join(
        f'<div class="ligne"><span class="pl">{i+1}</span>'
        f'<span class="ps">{j}</span>'
        f'<span class="bg">{svg(t, 18, c)}</span>'
        f'<span class="pt">{88 - i * 17}</span></div>'
        for i, (t, (p, c), j) in enumerate(zip(
            traces, PALIERS, ['Δ Riolu', 'nocturne', 'pti_suisse', 'Xirdrak', 'zk']))
    )
    lignes.append('<div class="rangee"><p class="legende">En situation, '
                  'dans une vraie ligne de classement.</p>'
                  f'<div class="table">{rangs}</div></div>')

    morceaux.append(
        f'<section><h2><span class="lettre">{lettre}</span>{nom}</h2>'
        f'<p class="note">{note}</p>{"".join(lignes)}</section>'
    )

HTML = """<!doctype html>
<html lang="fr"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Cairn, planche d'essai des emblemes de palier</title>
<style>
  :root { color-scheme: dark; }
  * { box-sizing: border-box; }
  body {
    margin: 0; padding: 44px 44px 80px;
    background: #0A0C0F;
    color: #F4F6F8;
    font: 14px/1.55 -apple-system, 'Segoe UI', system-ui, sans-serif;
  }
  .entete { max-width: 760px; margin-bottom: 52px; }
  h1 { font-size: 23px; font-weight: 600; margin: 0 0 10px; letter-spacing: -0.01em; }
  .chapo { color: #9AA3AD; font-size: 13.5px; margin: 0; }
  .chapo b { color: #F4F6F8; font-weight: 600; }
  section { margin-bottom: 64px; padding-top: 30px; border-top: 1px solid rgba(255,255,255,0.08); }
  h2 { font-size: 17px; font-weight: 600; margin: 0 0 8px; display: flex; align-items: center; gap: 12px; }
  .lettre {
    display: inline-flex; align-items: center; justify-content: center;
    width: 25px; height: 25px; border-radius: 7px; font-size: 12px;
    background: rgba(32,227,178,0.14); color: #20E3B2; border: 1px solid rgba(32,227,178,0.4);
  }
  .note { color: #9AA3AD; font-size: 13px; margin: 0 0 26px; max-width: 700px; }
  .rangee { margin-bottom: 30px; }
  .legende { font-size: 11px; text-transform: uppercase; letter-spacing: 0.06em;
             color: #6F7883; margin: 0 0 12px; max-width: 700px; line-height: 1.7; }
  .legende b { color: #C9D1D9; }
  .epreuve { background: rgba(255,255,255,0.028); border-radius: 12px; padding: 20px 22px 14px; }
  .epreuve .legende { text-transform: none; letter-spacing: 0; font-size: 12px; }
  .cases { display: flex; align-items: flex-end; gap: 30px; flex-wrap: wrap; }
  .case { display: flex; flex-direction: column; align-items: center; gap: 9px; }
  .boite { display: flex; align-items: center; justify-content: center; }
  .nom { font-size: 11px; color: #838C96; }
  .table { max-width: 420px; }
  .ligne { display: grid; grid-template-columns: 26px minmax(0,1fr) 30px 46px;
           align-items: center; gap: 12px; height: 38px; padding: 0 10px; border-radius: 8px; }
  .ligne:nth-child(even) { background: rgba(255,255,255,0.02); }
  .pl { color: #6F7883; font-size: 12px; text-align: right; }
  .ps { font-size: 13px; }
  .bg { display: flex; }
  .pt { font-size: 12px; color: #9AA3AD; text-align: right; }
  .bilan p { color: #9AA3AD; font-size: 13.5px; max-width: 720px; margin: 0 0 15px; }
  .bilan p b { color: #F4F6F8; font-weight: 600; }
  .bilan .gros { color: #C9D1D9; font-size: 15px; line-height: 1.6; }
  .bilan .lettre { background: rgba(255,200,87,0.14); color: #FFC857; border-color: rgba(255,200,87,0.4); }
</style></head><body>
<div class="entete">
  <h1>Emblemes de palier, planche d'essai</h1>
  <p class="chapo">Trois directions, cinq paliers, aux tailles ou ils serviront vraiment.
  Rien n'est integre : c'est une planche pour trancher. <b>Regarde d'abord la bande
  grisee de chaque direction</b>, celle sans couleur a 18 px. C'est la que les familles
  d'emblemes meurent, pas en grand.</p>
</div>
__CORPS__
</body></html>
"""


CONCLUSION = """
<section class="bilan">
  <h2><span class="lettre">?</span>Ce que la planche apprend</h2>
  <p class="gros">Aucune des trois ne passe entierement le test des 18 px
  sans couleur, et l'endroit ou elles echouent est le meme pour les trois :
  <b>Platine et Diamant se confondent</b>.</p>
  <p>C'est logique, et ca n'a rien a voir avec le talent du dessin. Une
  famille d'emblemes se lit par accumulation, et l'accumulation a une
  limite : au quatrieme echelon la silhouette est pleine, et le cinquieme
  n'a plus de place pour se distinguer sans sortir du cadre. J'ai essaye
  trois sorties differentes, la cle de voute qui devient cristal, un second
  hexagone autour du cube, un accent plein au lieu d'un accent creux. Les
  trois se voient a 40 px. Aucune ne se voit a 18.</p>
  <p><b>C'est exactement ce que disent les planches que tu m'as envoyees.</b>
  Elles portent toutes, en haut a droite, une seconde famille intitulee
  « simple versions, for menus + small use cases ». Un studio entier, avec
  un illustrateur a plein temps, a conclu que son embleme riche ne survivait
  pas aux petites tailles et en a dessine un second. Ce n'est donc pas un
  echec a rattraper, c'est la regle du genre.</p>
  <p>Deux sorties possibles, et elles ne coutent pas la meme chose.</p>
  <p><b>Un.</b> On assume que <b>la couleur identifie le palier en petit, et
  la forme a partir de 40 px</b>. C'est deja ce que fait le losange
  d'aujourd'hui, et ca marche : cinq teintes choisies pour se distinguer en
  peripherie du regard, personne ne confond le violet et le turquoise. La
  forme sert alors a dire « c'est un rang » et a faire son effet sur la
  carte de tete et le profil. Cout : zero, c'est le fonctionnement actuel.</p>
  <p><b>Deux.</b> On dessine une <b>seconde famille simplifiee</b> pour les
  colonnes, comme Blizzard. Deux jeux a tenir en accord au lieu d'un, donc
  deux fois le travail a chaque retouche, pour un gain qui ne se voit que
  si quelqu'un lit le classement en noir et blanc.</p>
  <p class="gros">Mon avis : la sortie un. Et donc le vrai critere pour
  choisir entre A, B et C n'est pas la lisibilite a 18 px, qui est reglee
  par la couleur. <b>C'est ce que tu veux voir en grand sur ton profil le
  jour ou tu montes de palier.</b></p>
</section>
"""

open('/home/claude/Cairn/docs/planche-paliers.html', 'w', encoding='utf-8').write(
    HTML.replace('__CORPS__', ''.join(morceaux) + CONCLUSION)
)
print('ecrit')
