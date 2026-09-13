# Epok Engine identity

The identity uses an angular, dimensional E with a detached diamond crossbar.
Brushed platinum faces, graphite sides and a restrained inner highlight sit on
a transparent background. The name is **Epok**, or **Epok Engine** in full.

- `epok-master.png`: square symbol artwork with genuine PNG alpha transparency.
- `epok.png`: 256 px RGBA icon embedded in the window and editor toolbar.
- `epok.ico`: Windows executable icon in 16–256 px sizes.
- `epok-lockup.png`: EPOK / ENGINE wordmark and symbol, embedded in the Hub hero.

The Hub displays the brand artwork immediately as part of its project browser,
without introducing an artificial delay. Both images are embedded in the binary
and work offline, regardless of the working directory.

Regenerate the packaged icon on Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools/build-app-icon.ps1
```

The master and lockup were produced with the built-in ImageGen tool from the
user's geometric E reference. The icon adaptation retains that shape; the lockup
adds the approved EPOK name. The packaging script resizes and encodes the master
without altering its design. The exact lockup prompt is in `generation-prompts.txt`.

Existing gameplay/editor illustrations in the documentation are historical
captures; freshly captured Epok Hub and editor screenshots show the new branding.
