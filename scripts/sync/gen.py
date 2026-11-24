#!/usr/bin/env python3
"""Builds src/i18n.rs and src/overlay.rs from Ronnie's Mathéo (its src/app/desktop.rs and src/i18n.rs).

Ronnie's repository next to this one (../bipbip), or its path in RONNIE_DIR. Mathéo's own parts (his
window, his menu, the update) are in overlay_head.rs, his season outfits in season.rs, the other plants he can be in skins.rs, his antics in antics.rs, his own words and hours in care.rs, next to this script: change them there, then run
`python3 scripts/sync/gen.py` and check with `cargo test` and `cargo clippy`.
"""
import re
from pathlib import Path

import os

HERE = Path(__file__).resolve().parent
OUT = HERE.parent.parent / "src"
RONNIE = Path(os.environ.get("RONNIE_DIR", HERE.parent.parent.parent / "bipbip"))

desk = (RONNIE / "src/app/desktop.rs").read_text().split("\n")
i18n = (RONNIE / "src/i18n.rs").read_text().split("\n")


def lines(a, b):
    """Lines a..b (1-based, inclusive)."""
    return "\n".join(desk[a - 1 : b])


def find(text, start=1):
    for i in range(start - 1, len(desk)):
        if desk[i].startswith(text):
            return i + 1
    raise SystemExit(f"not found: {text}")


# ---------- i18n
FIELDS = ["matheo_water", "matheo_moves", "matheo_shaking", "matheo_quiet", "matheo_ouch", "matheo_thanks",
          "matheo_thirsty", "matheo_bed", "matheo_weekend", "matheo_break", "matheo_hello", "matheo_hiding", "matheo_gone"]


def section(start_marker):
    i = next(k for k, l in enumerate(i18n) if l.startswith(start_marker))
    j = next(k for k in range(i + 1, len(i18n)) if i18n[k] == "};")
    return i18n[i + 1 : j]


def value(sec, name):
    for k, l in enumerate(sec):
        if l.startswith(f"    {name}: "):
            out = [l]
            m = k + 1
            while m < len(sec) and not re.match(r"^    [a-z_0-9]+: ", sec[m]):
                out.append(sec[m])
                m += 1
            return "\n".join(out)
    raise SystemExit(f"no {name}")


fr, en = section("pub static FR"), section("pub static EN")
MENU = {
    "menu_tips_on": ("Mes conseils", "My advice"),
    "menu_quiet": ("Me taire pendant 6 h", "Keep quiet for 6 hours"),
    "menu_speak": ("Reparler", "Speak again"),
    "menu_autostart": ("Lancer au démarrage", "Start with the computer"),
    "menu_lang": ("Language: English", "Langue : français"),
    "menu_page": ("Le site de Mathéo", "Mathéo's website"),
    "menu_version": ("Mathéo, version {v}", "Mathéo, version {v}"),
    "menu_quit": ("Quitter Mathéo", "Quit Mathéo"),
    "matheo_update": ("Une nouvelle version de moi est sortie ! Clique sur cette bulle pour l’installer, je me relance tout seul.",
                      "A new version of me is out! Click this bubble to install it, I’ll restart by myself."),
    "matheo_installing": ("Je me mets à jour… ne bouge pas.", "Updating myself… don’t move."),
    "matheo_updated": ("C’est installé, je reviens tout de suite !", "Installed, I’ll be right back!"),
    "matheo_update_failed": ("La mise à jour n’a pas marché. Je réessaierai plus tard.", "The update didn’t work. I’ll try again later."),
    "matheo_dock": ("Coucou ! Moi c’est Mathéo, ta nouvelle plante de bureau. Je veille sur toi, et toi tu m’arroses : marché conclu ? Pour que je me pose sur le Dock, autorise-moi dans Accessibilité (la fenêtre de macOS) ; sinon je reste juste au-dessus. Un clic droit sur moi pour mon menu !",
                    "Hi there! I’m Mathéo, your new desk plant. I look after you, you water me: deal? To stand on the Dock, allow me in Accessibility (macOS’ window); else I stay just above it. Right-click me for my menu!"),
    "menu_install": ("Installer la version {v}", "Install version {v}"),
    "menu_check": ("Rechercher une mise à jour", "Check for updates"),
    "menu_journal": ("Mon journal (en cas de souci)", "My log (if something's wrong)"),
    "matheo_checking": ("Je regarde s’il y a une nouvelle version de moi…", "Looking for a new version of me…"),
    "matheo_latest": ("Je suis à jour, c’est ma dernière version.", "I’m up to date, this is my latest version."),
    "matheo_unreachable": ("Je n’arrive pas à savoir s’il y a une nouvelle version. Réessaie plus tard.", "I can’t find out whether there’s a new version. Try again later."),
    "menu_installing": ("Mise à jour en cours…", "Updating…"),
    "menu_dock": ("Me poser sur le Dock…", "Stand on the Dock…"),
    "breathe_in": ("Inspire par le nez… {n}", "Breathe in through your nose… {n}"),
    "breathe_out": ("Souffle tout doucement par la bouche… {n}", "Breathe out slowly through your mouth… {n}"),
    "breathe_count": ("Respiration {k} sur {of}", "Breath {k} of {of}"),
    "matheo_ask_name": ("Au fait…\nComment je peux t’appeler ?\nClique dans la case et écris ton prénom :",
                        "By the way…\nWhat should I call you?\nClick in the box and type your name:"),
    "matheo_psst": ("Psst, {name}… ", "Psst, {name}… "),
    "menu_name": ("Mon prénom…", "My name…"),
    "name_hint": ("Ton prénom", "Your name"),
    "name_ok": ("C’est moi !", "That’s me!"),
    "name_later": ("Passer", "Skip"),
    "menu_hours": ("Mes horaires…", "My hours…"),
    "menu_skin": ("Mon apparence…", "My look…"),
    "menu_back": ("‹ Retour", "‹ Back"),
    "menu_hours_on": ("Mes rappels d’horaires", "My time reminders"),
    "menu_hour_morning": ("Pause du matin", "Morning break"),
    "menu_hour_lunch": ("Repas", "Lunch"),
    "menu_hour_afternoon": ("Pause de l’aprèm", "Afternoon break"),
    "menu_hour_home": ("Fin de journée", "End of the day"),
    "menu_hours_reset": ("Remettre par défaut", "Back to default"),
    "glass_one": ("{n} verre d’eau", "{n} glass of water"),
    "glass_many": ("{n} verres d’eau", "{n} glasses of water"),
    "thanks_glass_first": ("Premier verre d’eau de la journée !", "First glass of water today!"),
    "thanks_glass_more": ("Et de {glasses} aujourd’hui !", "That’s {glasses} today!"),
    "matheo_hi": ("Coucou ! Moi c’est Mathéo, ta nouvelle plante de bureau. Je veille sur toi : de l’eau, un peu de mouvement, des pauses. En échange, juste un petit coup d’arrosoir de temps en temps. Un clic droit sur moi pour mon menu !",
                  "Hi there! I’m Mathéo, your new desk plant. I look after you: water, a bit of moving, breaks. In return, just a little watering now and then. Right-click me for my menu!"),
}
V = {
    'matheo_pause': (['C’est l’heure de la pause, {name} ! Lève-toi, étire-toi, va prendre l’air : la pause, c’est important. Je garde l’écran, promis.', 'Pause ! Tes jambes ont envie de faire un tour, et ton cerveau d’un petit café. Vas-y, je ne touche à rien.', 'Hop hop hop, {name}, c’est la pause ! Même moi je fais la sieste entre deux arrosages. La pause, c’est sacré.', 'Ding ding ! C’est l’heure de souffler un peu. Cinq minutes loin de l’écran, et tu reviens tout neuf.'], ['Break time, {name}! Get up, stretch, get some fresh air: breaks matter. I’ll guard the screen, promise.', 'Break! Your legs want a little walk, and your brain a little coffee. Go on, I won’t touch a thing.', 'Hop hop hop, {name}, break time! Even I nap between waterings. Breaks are sacred.', 'Ding ding! Time to catch your breath. Five minutes away from the screen and you’ll be good as new.']),
    'matheo_lunch': (['C’est l’heure de manger ! Ton assiette t’attend (moi je me contente d’un verre d’eau, mais toi tu mérites mieux).', 'Miam, {name}, c’est l’heure du repas ! Et pas devant l’écran, hein : je surveille.', 'Ton ventre gargouille, je l’entends d’ici ! File manger, le travail ne s’envolera pas.', 'À table ! Un vrai repas, assis, sans écran. Moi je fais un peu de photosynthèse en t’attendant.'], ['Time to eat! Your plate is waiting (I’ll make do with a glass of water, but you deserve better).', 'Yum, {name}, lunchtime! And not in front of the screen, okay? I’m watching.', 'I can hear your tummy rumbling from here! Go eat, the work won’t fly away.', 'Lunch is served! A real meal, sitting down, no screen. I’ll do some photosynthesis while I wait.']),
    'matheo_home': (['C’est l’heure ! On ferme tout et on rentre à la maison. Tu as bien bossé, {name}, je suis fier de toi.', 'Fin de journée ! Range tes affaires, éteins tout, et va faire un truc qui te fait plaisir.', 'Et c’est fini pour aujourd’hui ! Le reste attendra demain, promis. Rentre bien, {name}.', 'Clap de fin ! Tu as fait de ton mieux, et c’est déjà beaucoup. Allez, ouste, à la maison !'], ['It’s time! Close everything and head home. You worked well, {name}, I’m proud of you.', 'End of the day! Pack up, switch off, and go do something you enjoy.', 'That’s it for today! The rest can wait until tomorrow, promise. Get home safe, {name}.', 'That’s a wrap! You did your best, and that’s already a lot. Off you go, home!']),
    'matheo_monday': (['Bon lundi, {name} ! On y va tout doux, une chose après l’autre. Moi je pousse à mon rythme, et ça marche.', 'Lundi matin… courage ! Un café, une petite liste, et on attaque la semaine tranquillement.', 'Coucou, nouvelle semaine ! J’ai un bon pressentiment pour celle-là, {name}.'], ['Happy Monday, {name}! Easy does it, one thing at a time. I grow at my own pace, and it works.', 'Monday morning… hang in there! A coffee, a little to-do list, and we start the week gently.', 'Hello, new week! I have a good feeling about this one, {name}.']),
    'matheo_lamp': (['Il fait déjà nuit… Allume une lampe ! Tes yeux te diront merci (et moi j’aime bien la lumière aussi).', 'Brr, il fait sombre ! Une petite lampe, et c’est tout de suite plus doux pour tes yeux.', 'Le soleil est parti se coucher. Allume la lumière, on n’est pas des chauves-souris !'], ['It’s already dark… Switch a lamp on! Your eyes will thank you (and I like light too).', 'Brr, it’s getting dark! A little lamp and it’s much gentler on your eyes.', 'The sun has gone to bed. Switch the light on, we’re not bats!']),
    'matheo_weekend_work': (['Psst, {name}… c’est le week-end. Tu es sûr de vouloir travailler ? Va jouer dehors, je garde la maison !', 'Euh… on est le week-end, tu sais ? Même le travail a besoin de se reposer.', 'Week-end détecté ! Si c’est pour le plaisir, d’accord. Sinon, file te reposer, {name}.'], ['Psst, {name}… it’s the weekend. Are you sure you want to work? Go play outside, I’ll mind the house!', 'Um… it’s the weekend, you know? Even work needs a rest.', 'Weekend detected! If it’s for fun, fine. Otherwise, go and rest, {name}.']),
    'matheo_eyes': (['Petite pause pour tes yeux : regarde au loin pendant 20 secondes. Par la fenêtre, c’est encore mieux !', 'Tes yeux fixent l’écran depuis un moment… Regarde un truc loin, très loin, pendant 20 secondes.', 'Jeu du jour : trouve l’objet le plus éloigné de toi et regarde-le 20 secondes. Tes yeux adorent ce jeu.', 'Cligne des yeux, regarde au loin, et souris. Voilà, tes yeux sont tout reposés (et moi je suis content).'], ['A little break for your eyes: look far away for 20 seconds. Out of the window is even better!', 'Your eyes have been staring at the screen for a while… Look at something far, far away for 20 seconds.', 'Game of the day: find the furthest thing from you and look at it for 20 seconds. Your eyes love this game.', 'Blink, look far away, and smile. There, your eyes are all rested (and I’m happy).']),
    'matheo_posture': (['Hop, on se redresse ! Le dos droit, les épaules en arrière… comme moi. Bon, moi j’ai un pot, c’est plus facile.', 'Tu ne serais pas un peu en train de te ratatiner, {name} ? Redresse-toi, comme une belle plante !', 'Check posture : pieds au sol, dos contre le dossier, épaules détendues. Parfait, on dirait moi.', 'Étire-toi vers le plafond, comme pour attraper le soleil. Ahhh, ça fait du bien, non ?'], ['Up we sit! Back straight, shoulders back… like me. Well, I have a pot, it’s easier.', 'Are you shrivelling up a little, {name}? Sit up straight, like a proud plant!', 'Posture check: feet on the floor, back against the chair, shoulders relaxed. Perfect, you look like me.', 'Stretch up to the ceiling, as if to catch the sun. Ahhh, feels good, doesn’t it?']),
    'matheo_breathe': (['On respire ensemble ? Quatre grandes respirations, suis-moi : je gonfle, tu inspires ; je dégonfle, tu souffles.', 'Petite pause respiration, {name} ? Regarde-moi : quand je gonfle, tu inspires ; quand je dégonfle, tu souffles.', 'Allez, quatre grandes bouffées d’air ensemble. Je montre l’exemple, tu me suis !'], ['Shall we breathe together? Four deep breaths, follow me: I swell, you breathe in; I shrink, you breathe out.', 'A little breathing break, {name}? Watch me: when I swell, breathe in; when I shrink, breathe out.', 'Come on, four big breaths of air together. I’ll lead, you follow!']),
    'breathe_done': (['Et voilà ! Tu te sens plus léger, non ? Moi j’ai l’impression d’avoir pris une feuille.', 'Bravo ! Quatre respirations, zéro stress. On refait ça quand tu veux.', 'Ahhh… tout calme. Tu vois, c’est facile d’être une plante.'], ['There you go! Feeling lighter? I feel like I grew a leaf.', 'Well done! Four breaths, zero stress. Let’s do it again whenever you like.', 'Ahhh… all calm. See, being a plant is easy.']),
    'matheo_hot': (['Il fait chaud ! Un grand verre d’eau pour toi, un petit pour moi ?', 'Ouf, quelle chaleur ! Bois un coup, {name}, avant de fondre comme une glace.', 'Mode canicule activé : de l’eau, de l’eau, et encore de l’eau !'], ['It’s hot! A big glass of water for you, a little one for me?', 'Phew, what a heat! Have a drink, {name}, before you melt like an ice cream.', 'Heatwave mode on: water, water, and more water!']),
    'matheo_back': (['Te revoilà, {name} ! Tu m’as manqué.', 'Ah, tu es là ! Je t’ai gardé ta place bien au chaud.', 'Coucou ! Je commençais à parler tout seul…', 'Re-bonjour, {name} ! On reprend où on en était ?'], ['There you are, {name}! I missed you.', 'Ah, you’re here! I kept your seat nice and warm.', 'Hi! I was starting to talk to myself…', 'Hello again, {name}! Shall we pick up where we left off?']),
    'matheo_named': (['Enchanté, {name} ! Je retiens, promis. Moi c’est toujours Mathéo, hein.', '{name} ! Quel joli prénom. Je vais le dire souvent, tu verras.'], ['Nice to meet you, {name}! I’ll remember, promise. I’m still Mathéo, by the way.', '{name}! What a lovely name. I’ll say it often, you’ll see.']),
    'matheo_again': (['Coucou, c’est encore moi ! Je me suis fait une petite beauté (nouvelle version), et j’ai appris plein de trucs.', 'Tadaa ! Une nouvelle version de moi, toute fraîche. Et j’ai des surprises pour toi.'], ['Hi, it’s me again! I got a little makeover (new version) and learned lots of new tricks.', 'Ta-da! A brand new version of me. And I have surprises for you.']),
    # What's new in this version, said once at the first start after the update. Changed at each
    # release, its words shown to and approved by Julien first.
    'matheo_news': (['Coucou, c’est moi ! J’ai appris plein de nouvelles phrases, et Noël approche… j’ai une petite surprise pour les fêtes !'],
                    ['Hi, it’s me! I learned lots of new things to say, and Christmas is coming… I have a little surprise for the holidays!']),
    # The evening: the glasses of water of the day (his waterings), 1 or 2, 3 to 5, 6 and more.
    'matheo_glasses_few': (['Petit bilan du jour, {name} : {glasses}. Demain, on en boit un de plus ?', 'Aujourd’hui : {glasses}. C’est un début ! Demain, on fait mieux ensemble ?'],
                           ['Today’s little review, {name}: {glasses}. Shall we have one more tomorrow?', 'Today: {glasses}. It’s a start! Shall we do better together tomorrow?']),
    'matheo_glasses_good': (['Bilan du jour : {glasses}, bien joué {name} ! Toi et mes racines, on est bien hydratés.', '{glasses} aujourd’hui ! Tu prends soin de toi, et de moi. Bravo !'],
                            ['Today’s review: {glasses}, well done {name}! You and my roots are nicely hydrated.', '{glasses} today! You look after yourself, and after me. Bravo!']),
    'matheo_glasses_lots': (['Waouh, {glasses} aujourd’hui ! Tu bois plus que moi, {name}.', 'Record ! {glasses} aujourd’hui. Je suis fier de toi.'],
                            ['Wow, {glasses} today! You drink more than I do, {name}.', 'A record! {glasses} today. I’m proud of you.']),
    # Some days: how are you? A thumb up or down in his bubble, and kind words either way.
    'matheo_mood': (['Au fait, {name}… ça va aujourd’hui ?', 'Petite question du jour : comment tu te sens ?', 'Hé, {name} ! Ta journée se passe bien ?', 'Coucou, {name} ! Petit point météo intérieure : plutôt soleil ou plutôt nuages ?', 'Je prends de tes nouvelles : comment ça se passe pour toi aujourd’hui ?', 'Dis-moi, {name}, la journée est plutôt douce ou plutôt rude ?'],
                    ['By the way, {name}… how are you today?', 'Little question of the day: how are you feeling?', 'Hey, {name}! Is your day going well?', 'Hi, {name}! Little inner weather check: more sunshine or more clouds?', 'Just checking in: how’s it going for you today?', 'Tell me, {name}, is the day going gently or roughly?']),
    'matheo_mood_up': (['Super ! Ça me fait pousser une feuille de plus.', 'Génial ! Garde ce sourire, il te va bien, {name}.', 'Trop bien ! Moi aussi je suis en pleine forme : on fait une belle équipe.', 'Ah, ça fait plaisir ! Profite bien de cette belle journée.', 'Youpi ! Ta bonne humeur, c’est mon soleil du jour.', 'Ça se voit, tu rayonnes ! Continue comme ça, {name}.', 'Parfait ! Je fais une petite danse des feuilles pour fêter ça.', 'Chouette ! Retiens ce qui t’a fait du bien aujourd’hui, on en refera.'],
                       ['Great! That makes me grow one more leaf.', 'Brilliant! Keep that smile, it suits you, {name}.', 'Awesome! I’m in great shape too: we make a fine team.', 'Ah, that’s lovely to hear! Enjoy this good day.', 'Yay! Your good mood is my sunshine today.', 'It shows, you’re glowing! Keep it up, {name}.', 'Perfect! I’m doing a little leaf dance to celebrate.', 'Lovely! Remember what did you good today, we’ll do it again.']),
    'matheo_mood_down': (['Besoin de te défouler, {name} ? Attrape-moi et envoie-moi valser : je rebondis, ça ne me fait même pas mal !', 'Courage. Prends cinq minutes pour toi, un verre d’eau, un peu d’air : tu le mérites.', 'Merci de me le dire. Vas-y doucement aujourd’hui, tu fais déjà de ton mieux.', 'Des jours sans, même les plantes en ont. Demain, on repousse ensemble.', 'Je t’envoie un gros câlin de feuilles. Ça va aller, {name}.', 'Ça arrive à tout le monde. Une chose à la fois, et un peu de douceur pour toi.', 'Je reste à côté de toi, {name}. Si tu veux, on respire un grand coup ensemble.', 'Même un jour gris finit par passer. Et moi, je trouve que tu t’en sors très bien.'],
                         ['Need to let off steam, {name}? Grab me and send me flying: I bounce, it doesn’t even hurt!', 'Hang in there. Take five minutes for yourself, a glass of water, some air: you deserve it.', 'Thanks for telling me. Go easy today, you’re already doing your best.', 'Even plants have off days. Tomorrow, we grow again together.', 'Sending you a big leafy hug. It’ll be all right, {name}.', 'It happens to everyone. One thing at a time, and a little kindness for yourself.', 'I’m staying right beside you, {name}. If you like, we can take a deep breath together.', 'Even a grey day passes in the end. And I think you’re doing really well.']),
    # His season's outfit on: a word of the feast now and then (see `feast_due`). Christmas before the
    # 24th, then on the day.
    'matheo_christmas_soon': (['Mon bonnet de Noël est sorti ! Toi aussi, tu as fait ton sapin ?', 'La neige tombe sur mes feuilles… Ça sent bon le chocolat chaud, non ?', 'Petit rappel de lutin : pense à faire une pause entre deux listes de cadeaux.', 'Sur ma lettre au Père Noël, j’ai mis : de l’eau, du soleil, et toi en pleine forme.', 'Décembre, c’est chargé. Prends le temps de souffler, la magie de Noël n’est pas pressée.', 'Tu entends les grelots ? Non ? Moi non plus, mais j’ai mon bonnet, alors c’est tout comme.'],
                              ['My Santa hat is out! Have you put up your tree too?', 'Snow is falling on my leaves… Smells like hot chocolate, doesn’t it?', 'A little elf reminder: take a break between two gift lists.', 'On my letter to Santa, I wrote: water, sunshine, and you in great shape.', 'December is busy. Take time to breathe, Christmas magic is in no hurry.', 'Hear the sleigh bells? No? Me neither, but I’ve got my hat, so it’s the same thing.']),
    'matheo_christmas': (['Joyeux Noël, {name} ! J’espère que tu passes un moment tout doux.', 'C’est Noël ! Qu’est-ce que tu fais devant l’écran ? Va en profiter, je garde la maison.', 'Ho ho ho ! Le Père Noël est passé ? Moi, j’ai eu un flocon sur chaque feuille.'],
                         ['Merry Christmas, {name}! I hope you’re having a lovely, cosy time.', 'It’s Christmas! What are you doing at the screen? Go and enjoy it, I’ll mind the house.', 'Ho ho ho! Has Santa been? I got a snowflake on every leaf.']),
    'matheo_new_year': (['Nouvelle année, nouvelles feuilles ! Je te souhaite plein de belles choses, {name}.', 'Ma résolution : pousser un peu chaque jour. Et toi, c’est quoi la tienne ?', 'Confettis ! Une année de plus ensemble, ça se fête. Avec un verre… d’eau, bien sûr.', 'Cette année, je continue à veiller sur toi. Promis, juré, arrosé.'],
                        ['New year, new leaves! Wishing you lots of lovely things, {name}.', 'My resolution: grow a little every day. What’s yours?', 'Confetti! Another year together, let’s celebrate. With a glass… of water, of course.', 'This year, I’ll keep looking after you. Cross my heart, water my pot.']),
    'matheo_valentine': (['Joyeuse Saint-Valentin ! Aujourd’hui, dis à quelqu’un que tu l’aimes bien : un ami, ta famille, ou même toi.', 'J’ai mis mon plus beau nœud rouge. Je t’aime bien, tu sais, {name}.', 'Des cœurs partout ! Tu mérites autant d’amour que tu en donnes.'],
                         ['Happy Valentine’s Day! Today, tell someone you care about them: a friend, your family, or even yourself.', 'I’m wearing my best red bow. I like you a lot, you know, {name}.', 'Hearts everywhere! You deserve as much love as you give.']),
    'matheo_easter': (['Regarde mes oreilles de lapin ! Tu crois que ça me va ?', 'J’ai trouvé un œuf à côté de mon pot ! Il est en chocolat, tu crois ?', 'Bientôt Pâques ou déjà Pâques : le chocolat, c’est toujours une bonne idée. Avec modération !', 'Le printemps est là, et moi je fais le lapin. Et toi, tu sautes de joie, {name} ?'],
                      ['Look at my bunny ears! Do you think they suit me?', 'I found an egg next to my pot! Do you think it’s chocolate?', 'Easter soon or Easter already: chocolate is always a good idea. In moderation!', 'Spring is here and I’m being a bunny. Are you jumping for joy, {name}?']),
    'matheo_halloween': (['Bouh ! Je t’ai fait peur ? Non ? Pourtant, j’ai mon chapeau de sorcière…', 'Une chauve-souris tourne autour de moi. Je crois qu’on s’entend bien, maintenant.', 'Des bonbons ou un sort ! Bon, un verre d’eau, ça me va aussi.', 'C’est la semaine d’Halloween : même les citrouilles font des pauses.'],
                         ['Boo! Did I scare you? No? But I’ve got my witch’s hat…', 'A bat is flying around me. I think we get along now.', 'Trick or treat! Well, a glass of water will do too.', 'It’s Halloween week: even pumpkins take breaks.']),
    'more_ouch': (['Aïe !', 'Ouille, ma tête !', 'Aïeuh ! Doucement…', 'Bonk ! Je vois des étoiles…'], ['Ouch!', 'Ow, my head!', 'Ouchie! Gently…', 'Bonk! I see stars…']),
    'more_shaking': (['Waaaah ! Arrête !', 'Je vais avoir mal au cœur !', 'Pas si fort, j’ai la tête qui tourne !'], ['Waaaah! Stop it!', 'I’m going to be sick!', 'Not so hard, my head’s spinning!']),
    'more_thanks': (['Merci ! Santé ! Ça fait du bien.', 'Ahhh, délicieux ! Merci, {name}.', 'Glou glou glou… Je revis !', 'Merci ! Je sens que je vais pousser d’un coup.'], ['Thanks! Cheers! That feels good.', 'Ahhh, delicious! Thank you, {name}.', 'Glug glug glug… I’m alive again!', 'Thanks! I can feel myself growing.']),
    'more_thirsty': (['J’ai soif… et toi aussi, sûrement. Un grand verre d’eau, puis un coup d’arrosoir ?', 'Mes feuilles font la tête… Un peu d’eau pour moi, et un verre pour toi ?', 'Je suis tout sec, {name}… On boit un coup ensemble ?'], ["I'm thirsty… and so are you, surely. A big glass of water, then the watering can?", 'My leaves are sulking… A little water for me, and a glass for you?', 'I’m all dry, {name}… Shall we have a drink together?']),
}

# More words of his, added to Ronnie's lists (`FIELDS`) and to his own (`V`): French, English.
MORE = {
    'matheo_water': (['Glou glou : un grand verre d’eau, et tu repars comme une fleur.', 'Petit rappel de ta plante préférée : l’eau, c’est la vie. Un verre ?', 'Tu as la bouche sèche ? Moi, j’ai les feuilles sèches. On boit un coup tous les deux ?'],
                     ['Glug glug: a big glass of water, and you’ll be fresh as a daisy.', 'A little reminder from your favourite plant: water is life. A glass?', 'Dry mouth? I’ve got dry leaves. Shall we both have a drink?']),
    'matheo_moves': (['Lève les genoux, comme si tu marchais sur place. Dix fois, allez !', 'Remonte les épaules vers tes oreilles, tiens trois secondes… et relâche. Ahhh.', 'Va remplir ton verre un peu plus loin : des pas et de l’eau, deux en un.', 'Fais de grands cercles avec les bras, comme un moulin. Cinq dans chaque sens.', 'Un appel à passer ? Fais-le debout, en marchant un peu.'],
                     ['Lift your knees, as if marching on the spot. Ten times, go!', 'Shrug your shoulders up to your ears, hold three seconds… and let go. Ahhh.', 'Go and fill your glass a little further away: steps and water, two in one.', 'Make big circles with your arms, like a windmill. Five each way.', 'A call to make? Take it standing up, walking around a little.']),
    'matheo_bed': (['Les étoiles sont de sortie. Pour toi, c’est l’heure de la couette.', 'Un bon sommeil, c’est le meilleur des engrais. Bonne nuit !', 'Je replie mes feuilles pour la nuit. Fais pareil avec ton écran, à demain !'],
                   ['The stars are out. For you, it’s duvet time.', 'Good sleep is the best fertiliser. Good night!', 'I’m folding my leaves for the night. Do the same with your screen, see you tomorrow!']),
    'matheo_weekend': (['Vendredi ! Je sors déjà mon petit chapeau de paille.', 'Encore un petit effort, et c’est deux jours rien qu’à toi.', 'Tu as prévu quoi ce week-end ? Moi : soleil, eau fraîche et grasse matinée.'],
                       ['Friday! I’m already getting my little straw hat out.', 'One last little push, and it’s two days all for you.', 'What are your weekend plans? Mine: sun, fresh water and a lie-in.']),
    'matheo_break': (['Deux heures sans bouger… Même un cactus ferait une pause, tu sais.', 'Pause obligatoire, ordre de ta plante ! Va marcher un peu, je ne bouge pas d’ici.', 'Tes épaules ont besoin d’une pause, tes yeux aussi. Cinq minutes, debout !'],
                     ['Two hours without moving… Even a cactus would take a break, you know.', 'Mandatory break, plant’s orders! Go for a little walk, I’m not going anywhere.', 'Your shoulders need a break, and your eyes too. Five minutes, on your feet!']),
    'matheo_pause': (['Pause, {name} ! Un verre d’eau, quelques pas, un regard par la fenêtre : la recette parfaite.', 'Tu as bien avancé. Maintenant, cinq minutes pour toi, sans écran.', 'Pause méritée ! Je garde ta place au chaud en t’attendant.'],
                     ['Break time, {name}! A glass of water, a few steps, a look out of the window: the perfect recipe.', 'You’ve made good progress. Now, five minutes for you, no screen.', 'Well-earned break! I’ll keep your seat warm while you’re away.']),
    'matheo_lunch': (['Midi, l’heure de recharger les batteries ! Un bon repas, et tu reviens en pleine forme.', 'On mange, {name} ! Prends ton temps, mâche doucement, savoure.', 'Petite faim ? Grosse faim ? Dans tous les cas, c’est l’heure du repas !'],
                     ['Noon, time to recharge! A good meal, and you’ll be back in great shape.', 'Time to eat, {name}! Take your time, chew slowly, enjoy.', 'Peckish? Starving? Either way, it’s mealtime!']),
    'matheo_home': (['La journée est finie ! Déconnecte un peu, tu l’as bien mérité, {name}.', 'Bravo pour aujourd’hui. Maintenant, place au repos, à tes proches et aux bonnes choses.', 'Ferme tes onglets, range ton bureau, et à demain ! Je veille sur tout ici.'],
                    ['The day’s over! Switch off a bit, you’ve earned it, {name}.', 'Well done today. Now it’s time for rest, loved ones and good things.', 'Close your tabs, tidy your desk, and see you tomorrow! I’ll keep an eye on things here.']),
    'matheo_monday': (['C’est reparti pour une semaine ! On commence par un verre d’eau, le reste suivra.', 'Lundi, nouveau départ. Qu’est-ce qui te ferait plaisir cette semaine, {name} ?'],
                      ['Here we go, a new week! We start with a glass of water, the rest will follow.', 'Monday, a fresh start. What would make you happy this week, {name}?']),
    'matheo_lamp': (['Il fait nuit tôt en ce moment… Une lampe allumée, et l’ambiance devient tout de suite plus douce.', 'De la lumière, s’il te plaît ! Travailler dans le noir, ça fatigue les yeux.'],
                    ['It gets dark early these days… A lamp on, and things feel cosier right away.', 'Light, please! Working in the dark tires your eyes.']),
    'matheo_weekend_work': (['C’est le week-end, {name}… Une petite heure, et après on file en profiter ?', 'Le week-end, c’est fait pour recharger les batteries. Ne reste pas trop longtemps, d’accord ?'],
                            ['It’s the weekend, {name}… One little hour, and then off you go to enjoy it?', 'Weekends are for recharging. Don’t stay too long, all right?']),
    'matheo_eyes': (['Ferme les yeux dix secondes. Voilà… Ça fait du bien, non ?', 'La règle du 20-20-20 : toutes les 20 minutes, regarde à 6 mètres pendant 20 secondes. On essaie ?'],
                    ['Close your eyes for ten seconds. There… Feels good, doesn’t it?', 'The 20-20-20 rule: every 20 minutes, look 20 feet away for 20 seconds. Shall we try?']),
    'matheo_posture': (['L’écran à hauteur des yeux, les pieds bien à plat : voilà une installation de pro.'],
                       ['Screen at eye level, feet flat on the floor: now that’s a pro setup.']),
    'matheo_breathe': (['Une minute de calme, {name} ? Respire avec moi : je gonfle, tu inspires ; je dégonfle, tu souffles.', 'On fait une pause respiration ? Suis mes feuilles : quand je grossis, inspire ; quand je rapetisse, souffle.'],
                       ['A minute of calm, {name}? Breathe with me: I swell, you breathe in; I shrink, you breathe out.', 'A breathing break? Follow my leaves: when I grow, breathe in; when I shrink, breathe out.']),
    'breathe_done': (['Et voilà, quatre respirations ! Le calme est revenu.', 'Bien joué ! Garde ce calme avec toi pour la suite de la journée.'],
                     ['There we go, four breaths! Calm is back.', 'Well done! Keep that calm with you for the rest of the day.']),
    'matheo_hot': (['Grosse chaleur aujourd’hui ! Garde une bouteille d’eau à portée de main.', 'Il fait si chaud que mes feuilles transpirent. Bois, {name}, bois !'],
                   ['Big heat today! Keep a bottle of water within reach.', 'It’s so hot my leaves are sweating. Drink, {name}, drink!']),
    'matheo_back': (['Ah, te voilà ! Je me demandais où tu te cachais.', 'Bon retour, {name} ! On reprend tranquillement ?'],
                    ['Ah, there you are! I was wondering where you were hiding.', 'Welcome back, {name}! Shall we ease back in?']),
    'matheo_named': (['{name}, c’est noté ! Je le garde précieusement dans mes racines.'],
                     ['{name}, noted! I’ll keep it safe in my roots.']),
    'more_ouch': (['Ouïe ouïe ouïe…', 'Mes feuilles !'], ['Ow ow ow…', 'My leaves!']),
    'more_shaking': (['Stooop ! Je vais perdre mes feuilles !', 'Je ne suis pas un shaker, moi !'], ['Stooop! I’m going to lose my leaves!', 'I’m not a cocktail shaker!']),
    'more_thanks': (['Mmm, de l’eau bien fraîche ! Merci.', 'Merci ! Mes racines font la fête.'], ['Mmm, nice fresh water! Thanks.', 'Thanks! My roots are partying.']),
    'more_thirsty': (['Psst… ma terre est sèche comme un désert. Et ton verre, il en est où ?', 'Un peu d’eau, s’il te plaît ? On partage : un verre pour toi, l’arrosoir pour moi.'],
                     ['Psst… my soil is as dry as a desert. And how’s your glass doing?', 'A little water, please? Let’s share: a glass for you, the can for me.']),
    'matheo_glasses_few': (['Aujourd’hui, {glasses}. Pas grave ! Demain, on garde un verre à portée de main ?'], ['Today, {glasses}. No worries! Tomorrow, shall we keep a glass within reach?']),
    'matheo_glasses_good': (['{glasses} aujourd’hui, joli score ! Toi et moi, on pousse bien.'], ['{glasses} today, nice score! You and I are growing well.']),
    'matheo_glasses_lots': (['{glasses} ! Tu es une vraie source, {name}.'], ['{glasses}! You’re a real spring, {name}.']),
}
for k, (f, e) in MORE.items():
    if k in V:
        V[k] = (V[k][0] + f, V[k][1] + e)
    else:
        assert k in FIELDS, k

decl = "\n".join(
    [f"    pub {f}: &'static [&'static str]," if f in ("matheo_water", "matheo_moves", "matheo_bed", "matheo_weekend", "matheo_break")
     else f"    pub {f}: &'static str," for f in FIELDS]
    + [f"    pub {k}: &'static str," for k in MENU]
    + [f"    pub {k}: &'static [&'static str]," for k in V]
)


# Ronnie's words for developers, said otherwise by Mathéo: not only developers have him.
REWORD = [
    ("Il est tard… Va te coucher, le code sera encore là demain.", "Il est tard… Va te coucher, le travail sera encore là demain."),
    ("Le bug de ce soir, tu le trouveras en deux minutes demain matin. Au lit !", "Ce qui te bloque ce soir, tu le régleras en deux minutes demain matin. Au lit !"),
    ("Vendredi après-midi : on ne met rien en prod, d’accord ?", "Vendredi après-midi : on ne se lance pas dans un gros chantier, d’accord ?"),
    ("It's late… Go to bed, the code will still be there tomorrow.", "It's late… Go to bed, the work will still be there tomorrow."),
    ("Tonight's bug, you'll find it in two minutes tomorrow morning. Bedtime!", "What's stuck tonight, you'll sort out in two minutes tomorrow morning. Bedtime!"),
    ("Friday afternoon: nothing goes to production, all right?", "Friday afternoon: no big new project, all right?"),
]


def reworded(text):
    for a, b in REWORD:
        text = text.replace(a, b)
    return text


def more(text, field, idx):
    """Ronnie's list `field` (its Rust lines), with Mathéo's words in `MORE` added at its end."""
    if field not in MORE:
        return text
    head, end = text.rsplit("\n", 1)
    return head + "\n" + "".join(f'        "{x}",\n' for x in MORE[field][idx]) + end


def body(sec, idx):
    lists = [f"    {k}: &[\n" + "".join(f'        "{x}",\n' for x in v[idx]) + "    ]," for k, v in V.items()]
    return "\n".join([more(reworded(value(sec, f)), f, idx) for f in FIELDS] + [f'    {k}: "{v[idx]}",' for k, v in MENU.items()] + lists)


_said = "\n".join(value(fr, f) + value(en, f) for f in FIELDS)
for a, _ in REWORD:
    if a not in _said:
        raise SystemExit(f"no longer in Ronnie's words: {a}")


(OUT / "i18n.rs").write_text(f'''//! Mathéo's words, in French and in English (taken from Ronnie's).

use serde::{{Deserialize, Serialize}};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum Lang {{
    #[default]
    Fr,
    En,
}}

impl Lang {{
    /// Every language (the tests go through them).
    #[cfg(test)]
    pub const ALL: [Lang; 2] = [Lang::Fr, Lang::En];

    pub fn strings(self) -> &'static Strings {{
        match self {{
            Lang::Fr => &FR,
            Lang::En => &EN,
        }}
    }}

    /// The system's language: English when it says so, French otherwise.
    pub fn from_system() -> Self {{
        let lang = ["LC_ALL", "LC_MESSAGES", "LANG"].iter().find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty())).unwrap_or_default();
        if lang.starts_with("en") {{ Lang::En }} else {{ Lang::Fr }}
    }}

    pub fn other(self) -> Self {{
        match self {{
            Lang::Fr => Lang::En,
            Lang::En => Lang::Fr,
        }}
    }}
}}

pub struct Strings {{
{decl}
}}

pub static FR: Strings = Strings {{
{body(fr, 0)}
}};

pub static EN: Strings = Strings {{
{body(en, 1)}
}};
''')

# ---------- overlay
other_from = find("/// A window over another screen than the pointer's.")
plant_to = find("pub(crate) fn run() {") - 1
plant = lines(other_from, plant_to)
plant = plant.replace("crate::config::config_dir()", "crate::config_dir()")
plant = "\n".join(l for l in plant.split("\n") if "`ronnie desktop-fx`" not in l)
plant = plant.replace('std::env::var("RONNIE_MATHEO_CLOCK")', 'std::env::var("MATHEO_CLOCK")')
plant = plant.replace("/// Debug builds: `RONNIE_MATHEO_CLOCK=late|friday`", "/// Debug builds: `MATHEO_CLOCK=late|friday`")
plant = plant.replace("/// Debug builds: `RONNIE_MATHEO_GALLERY=1`", "/// Debug builds: `MATHEO_GALLERY=1`")
for gone in ['        ("Salut Gérard", Look { wave: 1.0, ..base }, None),\n', '        ("La horde, 10 h", Look { hide: 1.0, ..base }, None),\n']:
    assert gone in plant, gone
    plant = plant.replace(gone, "")
plant = plant.replace("pub(super) fn paint_matheo(", "#[allow(dead_code)]\nfn paint_matheo(")
# His eyes following the pointer near him (`gaze`, set by the window: see overlay_head.rs).
for a, b in [
    ("    wilt: f32,\n}\n", "    wilt: f32,\n    /// Where his eyes look, the pointer near him (his units, 1 at most): towards it.\n    gaze: Vec2,\n    /// Which plant he is (see `skins.rs`).\n    skin: Skin,\n    /// Changing plant: how far down in the soil (0 out, 1 gone; below 0, popping out bigger).\n    sink: f32,\n    /// The season's outfit he wears: Paloma and Hélio add a touch of their own (see `skins.rs`).\n    feast: Option<Season>,\n}\n"),
    ("            flower: self.growth.2 >= DAYS_A_FLOWER,\n", "            flower: self.growth.2 >= DAYS_A_FLOWER,\n            gaze: Vec2::ZERO,\n            skin: Skin::Classic,\n            sink: 0.0,\n            feast: None,\n"),
    ("        } else {\n            painter.circle_filled(at(x, y), 2.0 * u, ink);\n",
     "        } else {\n            // Following the pointer near him.\n            painter.circle_filled(at(x + look.gaze.x * 1.4, y + look.gaze.y * 1.1), 2.0 * u, ink);\n"),
]:
    assert plant.count(a) == 1, a
    plant = plant.replace(a, b)
# Another plant than him (a cactus, a sunflower: see skins.rs): drawn instead of his leaves and his flower.
leaves_from = "    // His leaves: stems from the soil, big leaves swaying (flapping, held).\n"
leaves_to = "    // Tired: z's floating up from his leaves.\n"
assert plant.count(leaves_from) == 1 and plant.count(leaves_to) == 1
i, j = plant.index(leaves_from), plant.index(leaves_to)
inner = "".join(("        " + l if l.strip() else l) for l in plant[i:j].splitlines(keepends=True))
plant = (plant[:i]
         + "    // Changing plant (see skins.rs): sinking into the soil, popping out.\n"
         + "    let grown = 1.0 - look.sink;\n"
         + "    let u = u * grown;\n"
         + "    let at = |x: f32, y: f32| at(x * grown, -32.0 + (y + 32.0) * grown);\n"
         + "    if grown > 0.02 {\n"
         + "        if look.skin != Skin::Classic {\n            paint_skin(painter, look, now, u, &at);\n        } else {\n"
         + inner + "        }\n    }\n" + plant[j:])
# His flower of the day: open once watered GOOD_DAY times today, gone the next day (not after 20 good days).
for x, y in [("            flower: self.growth.2 >= DAYS_A_FLOWER,\n",
              "            // Today's flower (the date only looked at once watered enough).\n"
              "            flower: self.growth.1 >= GOOD_DAY && self.growth.0 == chrono::Local::now().format(\"%Y-%m-%d\").to_string(),\n"),
             ("        // Grown for a month: a pink flower", "        // Watered well today: a pink flower")]:
    assert plant.count(x) == 1, x
    plant = plant.replace(x, y)
# A word when a newer version is out, and hello at the very first start.
plant = plant.replace("""    /// The horde gone.
    Gone,
}""", """    /// The horde gone.
    Gone,
    /// A newer version of him published.
    Update,
    /// Hello, the very first time.
    Hi,
    /// Hello, the very first time on macOS, asking for Accessibility (to stand on the Dock).
    Dock,
    Installing,
    Updated,
    UpdateFailed,
    /// Looking for a newer version, asked from his menu.
    Checking,
    /// Asked: he's the latest version.
    Latest,
    /// Asked: floor couldn't be reached.
    Unreachable,
    /// What he says of his own (see `care.rs`).
    Own(Own),
}""")
plant = plant.replace("""            Said::Hiding => Duration::from_millis(1500),""", """            Said::Hiding => Duration::from_millis(1500),
            Said::Update | Said::Hi | Said::Dock => Duration::from_secs(30),
            Said::Installing => Duration::from_secs(120),
            Said::Updated => Duration::from_secs(5),
            Said::UpdateFailed | Said::Latest | Said::Unreachable => Duration::from_secs(8),
            Said::Checking => Duration::from_secs(40),
            Said::Own(o) => o.lasts(),""")
plant = plant.replace("""            Said::Gone => t.matheo_gone,""", """            Said::Gone => t.matheo_gone,
            Said::Update => t.matheo_update,
            Said::Hi => t.matheo_hi,
            Said::Dock => t.matheo_dock,
            Said::Installing => t.matheo_installing,
            Said::Updated => t.matheo_updated,
            Said::UpdateFailed => t.matheo_update_failed,
            Said::Checking => t.matheo_checking,
            Said::Latest => t.matheo_latest,
            Said::Unreachable => t.matheo_unreachable,
            Said::Own(o) => o.text(t, 0),""")
assert "Said::Update" in plant and "t.matheo_hi" in plant

ui_from = find("        // Mathéo: grabbed with the mouse")
gerard = find("            // Gérard passing near him", ui_from)
pass_line = find("            let pass = !(over || plant.grab.is_some());", ui_from)
block = lines(ui_from, gerard - 1)
block = block.replace("        if self.shown.matheo {\n", "        {\n", 1)
block = block.replace("self.shown.matheo_tips", "self.settings.tips")
# The watering can kept beside him until used (not gone after 3 min): thirstier once, when it has
# waited CAN_WAITS (see `can_counted`); drawn again only while it hops in or wiggles.
for x, y in [
    ("""                Some((since, None)) if since.elapsed() >= CAN_WAITS => {
                    plant.can = None;
                    plant.set_thirst(plant.thirst + 1);
                }
""", """                Some((since, None)) if since.elapsed() >= CAN_WAITS && self.can_counted != Some(since) => {
                    self.can_counted = Some(since);
                    plant.set_thirst(plant.thirst + 1);
                }
                Some((since, None)) => {
                    let t = since.elapsed().as_secs_f32();
                    if t < 0.6 || t % 3.0 < 0.7 {
                        plant_busy = true;
                    }
                }
"""),
]:
    assert block.count(x) == 1, x
    block = block.replace(x, y)
# A right click on him (or his bubble): his menu.
anchor = "            let over = mouse.at.is_some_and(|p| body.contains(p)) || on_bubble || on_can;\n"
assert anchor in block
block = block.replace(anchor, anchor + """            if right_click && (mouse.at.is_some_and(|p| body.contains(p)) || on_bubble) {
                open_menu = mouse.at;
            }
""")
assert block.count("click") > 3

os_from = find('#[cfg(target_os = "macos")]', pass_line)
tests_at = find("#[cfg(test)]", os_from)
oses = lines(os_from, tests_at - 1)
# The right button too.
reps = [
    ("Mouse { at: Some(Pos2::new(p.x as f32, p.y as f32)), pressed: CGEventSourceButtonState(0, 0) }",
     "Mouse { at: Some(Pos2::new(p.x as f32, p.y as f32)), pressed: CGEventSourceButtonState(0, 0), right: CGEventSourceButtonState(0, 1) }"),
    ("let (ok, pressed) = unsafe { (GetCursorPos(&mut p) != 0, (GetAsyncKeyState(VK_LBUTTON as i32) as u16 & 0x8000) != 0) };",
     "let (ok, pressed, right) = unsafe { (GetCursorPos(&mut p) != 0, (GetAsyncKeyState(VK_LBUTTON as i32) as u16 & 0x8000) != 0, (GetAsyncKeyState(VK_RBUTTON as i32) as u16 & 0x8000) != 0) };"),
    ("Mouse { at: Some(Pos2::new(p.x as f32 / scale, p.y as f32 / scale)), pressed }",
     "Mouse { at: Some(Pos2::new(p.x as f32 / scale, p.y as f32 / scale)), pressed, right }"),
    ("use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};",
     "use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON};"),
    ("Some(p) => Mouse { at: Some(Pos2::new(f32::from(p.root_x), f32::from(p.root_y))), pressed: p.mask.contains(KeyButMask::BUTTON1) },",
     "Some(p) => Mouse { at: Some(Pos2::new(f32::from(p.root_x), f32::from(p.root_y))), pressed: p.mask.contains(KeyButMask::BUTTON1), right: p.mask.contains(KeyButMask::BUTTON3) },"),
    ('"Ronnie\'s rewards over the screen"', '"Mathéo over the screen"'),
    # Windows: the frame left to DWM (see the comment put in its place).
    ("use windows_sys::Win32::Graphics::Dwm::{DWMNCRP_DISABLED, DWMWA_BORDER_COLOR, DWMWA_NCRENDERING_POLICY, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND, DwmSetWindowAttribute};",
     "use windows_sys::Win32::Graphics::Dwm::{DWMWA_BORDER_COLOR, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND, DwmSetWindowAttribute};"),
    ("            set(DWMWA_NCRENDERING_POLICY, DWMNCRP_DISABLED as u32);\n",
     "            // Not DWMNCRP_DISABLED: winit's windows keep WS_CAPTION | WS_BORDER (their frame only\n"
     "            // trimmed through WM_NCCALCSIZE), and without DWM drawing the frame, DefWindowProc painted\n"
     "            // a classic one over the whole window at each WM_NCACTIVATE (him taking or losing the\n"
     "            // focus): a screen-sized frame around nothing.\n"),
    # Linux: out of the task bar written before shown, asked again when lost (see the comment put in its place).
    ('    /// Every window of ours out of the task bar and the pager, on every desktop (asked of the window\n    /// manager once each, when it lists them).\n    pub(super) fn tune_all() {\n        static DONE: std::sync::Mutex<Vec<Window>> = std::sync::Mutex::new(Vec::new());\n        let Some((conn, root)) = x() else { return };\n        let me = std::process::id();\n        let (Some(state), Some(desktop), Some(skip_bar), Some(skip_pager), Some(sticky)) = (atom(conn, "_NET_WM_STATE"), atom(conn, "_NET_WM_DESKTOP"), atom(conn, "_NET_WM_STATE_SKIP_TASKBAR"), atom(conn, "_NET_WM_STATE_SKIP_PAGER"), atom(conn, "_NET_WM_STATE_STICKY")) else { return };\n        let Ok(mut done) = DONE.lock() else { return };\n        let mask = EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY;\n        for w in values(conn, *root, "_NET_CLIENT_LIST") {\n            if done.contains(&w) || values(conn, w, "_NET_WM_PID").first() != Some(&me) {\n                continue;\n            }\n            // 1: add; then the source, a normal application.\n            let _ = conn.send_event(false, *root, mask, ClientMessageEvent::new(32, w, state, [1, skip_bar, skip_pager, 1, 0]));\n            let _ = conn.send_event(false, *root, mask, ClientMessageEvent::new(32, w, state, [1, sticky, 0, 1, 0]));\n            let _ = conn.send_event(false, *root, mask, ClientMessageEvent::new(32, w, desktop, [0xFFFF_FFFF, 1, 0, 0, 0]));\n            done.push(w);\n        }\n        let _ = conn.flush();\n    }\n',
     '    /// Every window of ours out of the task bar and the pager, above, on every desktop. Written on\n    /// the window while it is not shown yet (the window manager reads it as it takes the window in;\n    /// some docks never look again: a window per screen listed as an open app), then asked of the\n    /// window manager each second for those shown without it (a window hidden then shown loses it).\n    pub(super) fn tune_all() {\n        use std::time::{Duration, Instant};\n\n        use x11rb::protocol::xproto::{MapState, PropMode};\n        use x11rb::wrapper::ConnectionExt as _;\n\n        /// The root\'s children looked at, the listed windows not ours, those asked again, when last.\n        type Seen = (Vec<Window>, Vec<Window>, Vec<Window>, Option<Instant>);\n        static SEEN: std::sync::Mutex<Seen> = std::sync::Mutex::new((Vec::new(), Vec::new(), Vec::new(), None));\n        let Some((conn, root)) = x() else { return };\n        let me = std::process::id();\n        let names = ["_NET_WM_STATE", "_NET_WM_DESKTOP", "_NET_WM_STATE_SKIP_TASKBAR", "_NET_WM_STATE_SKIP_PAGER", "_NET_WM_STATE_STICKY", "_NET_WM_STATE_ABOVE"];\n        let Some([state, desktop, skip_bar, skip_pager, sticky, above]) = names.map(|n| atom(conn, n)).into_iter().collect::<Option<Vec<_>>>().and_then(|v| <[u32; 6]>::try_from(v).ok()) else { return };\n        let Ok(mut seen) = SEEN.lock() else { return };\n        let (children, strangers, asked, last) = &mut *seen;\n        // Ours not shown yet (right under the root: a shown one is in its frame).\n        if let Some(tree) = conn.query_tree(*root).ok().and_then(|c| c.reply().ok()) {\n            children.retain(|w| tree.children.contains(w));\n            for &w in &tree.children {\n                if children.contains(&w) {\n                    continue;\n                }\n                children.push(w);\n                if values(conn, w, "_NET_WM_PID").first() != Some(&me) {\n                    continue;\n                }\n                if conn.get_window_attributes(w).ok().and_then(|c| c.reply().ok()).is_some_and(|a| a.map_state == MapState::UNMAPPED) {\n                    let _ = conn.change_property32(PropMode::REPLACE, w, state, AtomEnum::ATOM, &[skip_bar, skip_pager, above, sticky]);\n                    let _ = conn.change_property32(PropMode::REPLACE, w, desktop, AtomEnum::CARDINAL, &[0xFFFF_FFFF]);\n                }\n            }\n        }\n        // Ours shown without it: asked of the window manager.\n        if last.is_none_or(|t| t.elapsed() >= Duration::from_secs(1)) {\n            *last = Some(Instant::now());\n            let listed = values(conn, *root, "_NET_CLIENT_LIST");\n            strangers.retain(|w| listed.contains(w));\n            let mask = EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY;\n            for w in listed {\n                if strangers.contains(&w) {\n                    continue;\n                }\n                if values(conn, w, "_NET_WM_PID").first() != Some(&me) {\n                    strangers.push(w);\n                    continue;\n                }\n                if values(conn, w, "_NET_WM_STATE").contains(&skip_bar) {\n                    continue;\n                }\n                if !asked.contains(&w) {\n                    asked.push(w);\n                    crate::log::line(&format!("fenêtre {w:#x} dans la barre des tâches : sortie demandée"));\n                }\n                // 1: add; then the source, a normal application.\n                let _ = conn.send_event(false, *root, mask, ClientMessageEvent::new(32, w, state, [1, skip_bar, skip_pager, 1, 0]));\n                let _ = conn.send_event(false, *root, mask, ClientMessageEvent::new(32, w, state, [1, sticky, 0, 1, 0]));\n                let _ = conn.send_event(false, *root, mask, ClientMessageEvent::new(32, w, desktop, [0xFFFF_FFFF, 1, 0, 0, 0]));\n            }\n        }\n        let _ = conn.flush();\n    }\n'),
]
for a, b in reps:
    assert a in oses, a
    oses = oses.replace(a, b)

tests_from = tests_at
tests = lines(tests_from, len(desk))
keep = ["matheo_lands_at_the_foot_of_his_screen", "matheo_lands_on_the_screen_under_him_when_screens_are_stacked", "mathéo_says_water_and_moving_in_turns", "the_can_stands_on_the_free_side_and_comes_back_down", "mathéo_wilts_with_thirst"]
fns = re.split(r"\n(?=    #\[test\]\n)", tests)
head, rest = fns[0], fns[1:]
kept = [f for f in rest if any(f"fn {k}(" in f for k in keep)]
assert len(kept) == len(keep), [k for k in keep if not any(f"fn {k}(" in f for f in kept)]
tests = "\n".join([head] + kept).rstrip()
if not tests.endswith("}"):
    tests += "\n}"
tests = re.sub(r"\n    use [^\n]*Pointer[^\n]*", "", tests)
tests = tests.replace("crate::i18n::Lang::ALL", "crate::i18n::Lang::ALL")

overlay = (HERE / "overlay_head.rs").read_text()
overlay = overlay.replace("//@SEASON@", (HERE / "season.rs").read_text().rstrip())
overlay = overlay.replace("//@SKINS@", (HERE / "skins.rs").read_text().rstrip())
overlay = overlay.replace("//@ANTICS@", (HERE / "antics.rs").read_text().rstrip())
overlay = overlay.replace("//@CARE@", (HERE / "care.rs").read_text().rstrip())
overlay = overlay.replace("//@PLANT@", plant).replace("//@BLOCK@", block).replace("//@OS@", oses).replace("//@TESTS@", tests)
(OUT / "overlay.rs").write_text(overlay)
print("ok", len(overlay.split(chr(10))), "lines")
