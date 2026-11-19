#!/usr/bin/env bash
# Mathéo's smoke test on Linux (.github/workflows/smoke.yml): a virtual X server with a window manager
# and a compositor, Mathéo started on it, his journal read to find him, hovered, thrown, his menu
# opened, the pointer moved away, then the gallery of his looks; a screenshot at each step.
# Everything in smoke-out/ (screenshots, journal). Needs Xvfb, openbox, picom, xdotool, imagemagick.
set -u
exe=${1:-target/debug/matheo}
out=smoke-out
mkdir -p "$out"

export DISPLAY=:99
Xvfb :99 -screen 0 1600x900x24 +extension Composite >"$out/xvfb.log" 2>&1 &
sleep 2
xsetroot -solid '#3a6ea5'
openbox >"$out/openbox.log" 2>&1 &
sleep 1
picom --backend xrender >"$out/picom.log" 2>&1 &
sleep 1
# A window under him, to see through his.
xterm -geometry 120x30+40+40 -e 'top' &
sleep 1

shot() {
	import -window root "$out/$1.png" && echo "screenshot $1"
}

# Where he stands (his pot's foot), the journal's last word on it.
foot() {
	local log
	log=$(find "$HOME/.config" -name matheo.log -path '*atheo-dev*' 2>/dev/null | head -1)
	[ -n "$log" ] || return 1
	grep -oE 'Some\(\((-?[0-9.]+), (-?[0-9.]+)\)\) ; ses' "$log" | tail -1 | sed -E 's/Some\(\((-?[0-9.]+), (-?[0-9.]+)\)\).*/\1 \2/' | awk '{printf "%d %d\n", $1, $2}'
}

xdotool mousemove 200 200
shot 00-desktop

"$exe" >"$out/stderr.log" 2>&1 &
pid=$!
sleep 10
shot 01-started
kill -0 "$pid" 2>/dev/null || echo "Mathéo exited: $(wait "$pid"; echo $?)"
# His windows as X sees them: mapped or not, where, their depth (32: with alpha), their properties.
xwininfo -root -tree >"$out/x-tree.txt" 2>&1
for id in $(grep -i 'math' "$out/x-tree.txt" | awk '{print $1}'); do
	{
		echo "=== $id"
		xwininfo -id "$id"
		xprop -id "$id"
	} >>"$out/x-windows.txt" 2>&1
done

if read -r x y < <(foot); then
	echo "foot: $x $y"
	y=$((y - 15))
	xdotool mousemove "$x" "$y"
	sleep 1
	shot 02-hover
	# Grabbed, thrown up and to the left.
	xdotool mousedown 1
	sleep 0.2
	for i in $(seq 1 12); do
		xdotool mousemove $((x - i * 12)) $((y - i * 25))
		sleep 0.025
	done
	xdotool mouseup 1
	sleep 0.25
	shot 03-thrown
	sleep 5
	shot 04-landed
	if read -r x y < <(foot); then
		echo "foot after the throw: $x $y"
		xdotool mousemove "$x" $((y - 15))
		sleep 0.5
		# Held a little: he reads the buttons 60 times a second.
		xdotool mousedown 3
		sleep 0.15
		xdotool mouseup 3
		sleep 1
		shot 05-menu
		xdotool mousemove 300 300 mousedown 1
		sleep 0.15
		xdotool mouseup 1
		sleep 1
	fi
else
	echo "no foot in the journal"
fi
xdotool mousemove 100 100
sleep 2
shot 06-pointer-away

kill "$pid" 2>/dev/null
sleep 1
log=$(find "$HOME/.config" -name matheo.log -path '*atheo-dev*' 2>/dev/null | head -1)
[ -n "$log" ] && cp "$log" "$out/matheo.log"

MATHEO_GALLERY=1 "$exe" >"$out/stderr-gallery.log" 2>&1 &
pid=$!
sleep 8
shot 07-gallery
kill "$pid" 2>/dev/null
sleep 1
[ -n "$log" ] && cp "$log" "$out/matheo-gallery.log"

# The same through OpenGL instead of Vulkan.
WGPU_BACKEND=gl MATHEO_GALLERY=1 "$exe" >"$out/stderr-gl.log" 2>&1 &
pid=$!
sleep 8
shot 08-gallery-gl
kill "$pid" 2>/dev/null
sleep 1
[ -n "$log" ] && cp "$log" "$out/matheo-gl.log"
exit 0
