# Changelog

Les nouveautés de chaque version de Mathéo. La section d'une version sert aussi de notes à sa page de téléchargement.

## [Unreleased]

## [0.2.0] - 2026-11-19

- **Ton prénom** : il te demande comment t’appeler, directement dans sa bulle (clique dans la case, écris, « C’est moi ! » ou « Passer »). Il le retient et le glisse de temps en temps dans ses phrases. Modifiable dans son menu (« Mon prénom… »).
- **Il se présente** à son premier lancement, puis te demande ton prénom ; après cette mise à jour, il te dit un petit coucou avant de le demander.
- **Un parachute** : lâché ou lancé de haut, il l’ouvre de temps en temps et descend doucement en se balançant ; au sol, la voile retombe à côté de lui.
- **Plus vivant quand on le lance** : les yeux ronds en plein vol, écrasé puis rebondissant comme de la gelée à l’atterrissage, un nuage de poussière quand il tombe fort, et il glisse au sol penché en arrière.
- **Ses heures**, en semaine : la pause à 10 h et à 16 h (« la pause, c’est important »), le repas à midi, et à 18 h, l’heure de rentrer. **Réglables dans son menu** (« Mes horaires… ») au quart d’heure près, ou coupées.
- **Il prend soin de toi** entre deux conseils, chacun son tour : tes yeux (regarder au loin), ta posture (il se redresse avec toi), une respiration guidée (quatre respirations, il gonfle quand tu inspires et dégonfle quand tu souffles, avec le décompte), et un grand verre d’eau quand il fait chaud l’été.
- **Selon le moment** : un mot doux le lundi matin, la lampe à allumer les soirs d’hiver, un « tu es sûr ? » si tu travailles le week-end, et il saute de joie quand tu reviens après plus d’une heure d’absence.
- **Jamais lassant** : chacune de ses phrases a plusieurs versions, tirées au hasard.

## [0.1.6] - 2026-11-19

- **Plusieurs écrans** : plus aucune fenêtre ne bouge. Chaque écran a la sienne, faite une fois à sa taille et posée en pixels (sous Windows, par Windows lui-même) ; elle montre Mathéo, sa bulle ou son menu quand ils y sont. Avant, une grande fenêtre suivait la souris d’écran en écran : avec des écrans à des échelles différentes (125 % ici, 100 % là), des fenêtres apparaissaient et disparaissaient.

- **Son journal** reste léger : 1 Mo au plus (puis il repart à zéro, l’ancien gardé), et la souris seule n’y écrit plus rien.

## [0.1.5] - 2026-11-19

- **Linux, écran mis à l’échelle** (125 %, 150 %… comme sur Deepin) : il s’affiche ; sa fenêtre était plus petite que l’écran et il se tenait en dehors.

## [0.1.4] - 2026-11-19

- **Son journal**, dans son menu : ce qui s’est passé à son lancement (système, carte graphique, écrans) et ses fenêtres, pour comprendre un souci sur un ordinateur qu’on ne voit pas.
- **Linux** : il s’affiche enfin. Il était dessiné avec Vulkan dans une fenêtre restée vide ; il passe par OpenGL. Il a besoin de la bibliothèque `libxkbcommon-x11` (paquet `libxkbcommon-x11-0` sur Debian et Ubuntu) ; sans elle, il s’arrête dès le lancement, et son journal le dit.
- **Windows** : il ne disparaît plus quand on clique sur lui (sa propre fenêtre passée devant était prise pour un film en plein écran).
- **Son menu** passe de nouveau par-dessus sa bulle.

## [0.1.3] - 2026-11-19

- **Ses tenues de saison** : bonnet de Père Noël et neige du 1er au 25 décembre, chapeau de fête et confettis au Nouvel an, nœud rouge et cœurs à la Saint-Valentin, oreilles de lapin et œuf de Pâques la semaine de Pâques, chapeau de sorcière et chauve-souris pour Halloween.
- **Écrans l’un au-dessus de l’autre** : posé sur l’écran du bas, il y reste (il remontait tout seul sur celui du haut).
- **Plusieurs écrans** : il ne bouge plus du tout quand la souris passe d’un écran à l’autre.
- **Ronnie en plein écran** : Mathéo reste visible par-dessus. Il se cache toujours devant un film ou un jeu en plein écran.

## [0.1.2] - 2026-11-19

- **Rechercher une mise à jour**, dans son menu : il regarde tout de suite et dit s’il y a une nouvelle version, s’il est à jour ou s’il n’arrive pas à savoir. Sans rien demander, il regarde maintenant toutes les heures (au lieu de toutes les 6 h).
- **Plusieurs écrans** : Mathéo ne bouge plus d’un coup quand la souris passe d’un écran à l’autre.
- **Windows** : plus d’éclair blanc sur l’écran quand la souris change d’écran.

## [0.1.1] - 2026-11-19

- **Windows** : plus de grande fenêtre blanche sur l’écran où se trouve la souris. Le fond de ses fenêtres reste transparent quand il passe d’un écran à l’autre et quand la souris passe sur lui.

## [0.1.0] - 2026-11-18

Première version : Mathéo, la plante verte de Ronnie, en appli à part pour macOS, Windows et Linux.

- **Il veille sur toi** : toutes les demi-heures, une bulle te rappelle de boire un verre d’eau ou de bouger un peu (étirements, quelques pas, les yeux au loin), seulement quand tu es devant l’écran. Après 2 h sans pause, il insiste en sautillant.
- **L’arrosoir** : avec chaque rappel d’eau, un petit arrosoir vert. Un clic, il arrose Mathéo. Oublie-le et il dépérit ; bien arrosé, il grandit jusqu’à fleurir.
- **Le vendredi après-midi**, lunettes noires et week-end en vue ; **après 23 h**, il bâille et t’envoie te coucher.
- **Attrape-le, lance-le** (il rebondit sur les bords de l’écran), **secoue-le** pour qu’il se taise 6 h.
- **Sur le Dock** (macOS, avec l’autorisation Accessibilité, demandée au premier lancement), tout en bas de l’écran à côté.
- **Son menu, d’un clic droit sur lui** : sa version, ses conseils, le faire taire, le lancer au démarrage (activé d’office), la langue, son site, quitter.
- **Ses mises à jour** : quand une nouvelle version sort, il le dit ; un clic sur sa bulle l’installe et il se relance tout seul.
