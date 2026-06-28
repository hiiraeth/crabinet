# crabinet

A small, fast command-line tool written in Rust that sorts the files in a folder
into category subfolders by extension. Point it at a messy directory (like your
`Downloads`), give it a place to put the sorted output, and it moves each file
into a folder named for its type: `img`, `video`, `code`, and so on, with
anything unrecognized landing in `misc`.

It includes a `--dry-run` mode so you can preview exactly what would happen
before a single file is touched.

## How it works

- **Non-recursive.** Only the immediate contents of the input folder are sorted.
  Subfolders are left exactly where they are, untouched. A `report.pdf` gets
  moved; a `Projects/` folder does not.
- **Extension-based categories.** Each file's extension is matched (case-insensitively)
  against a single category table. The match decides which subfolder it goes to.
- **Best-effort moves.** If one file can't be moved (permissions, name collision,
  a cross-device issue), the tool logs the failure and continues with the rest
  rather than aborting the whole run.
- **Files without an extension are skipped** and reported, since there's nothing
  to categorize them by.

## Usage

```
crabinet <input folder> <output folder> [--dry-run]
```

- `<input folder>` — the directory to sort. Must already exist.
- `<output folder>` — where the category subfolders are created and files are
  moved into. Created automatically if it doesn't exist.
- `--dry-run` — optional. Print what *would* move, without creating folders or
  moving anything. Can be placed anywhere in the arguments.

### Examples

Preview first (recommended):

```
crabinet ~/Downloads ~/Downloads/organized --dry-run
```

Then run for real:

```
crabinet ~/Downloads ~/Downloads/organized
```

Dry-run output looks like:

```
organizing files...
would move proxmox-ve_9.1-1.iso -> /home/you/Downloads/organized/misc/
would move notes.md -> /home/you/Downloads/organized/txt/

would move 2 files, skipped 0 files
finished organizing files in 140.00µs
```

## Categories

Files are sorted into the following folders based on their extension. Anything
that doesn't match any category goes to `misc`.

| Folder        | Extensions |
|---------------|------------|
| `img`         | png, jpg, jpeg, webp, gif, bmp, tiff, tif, heic, heif, svg, ico, raw, cr2, nef, arw, rw2 |
| `video`       | mp4, mkv, mov, avi, wmv, flv, webm, m4v, mpg, mpeg, 3gp |
| `audio`       | mp3, wav, flac, aac, ogg, m4a, wma, aiff |
| `txt`         | txt, md, rtf, log, tex |
| `docs`        | pdf, doc, docx, xls, xlsx, ppt, pptx, odt, ods, odp, csv |
| `apps`        | dmg, pkg, app, exe, msi, deb, rpm, appimage, jar |
| `compressed`  | zip, rar, 7z, tar, gz, bz2, xz, tgz, zst |
| `code`        | rs, py, js, ts, c, cpp, h, java, go, rb, sh, json, toml, yaml, yml, html, glsl, ipynb |
| `misc`        | *(fallback for anything unmatched)* |

The category table is a single source of truth in the source: adding an extension
or a new category there automatically wires up both the folder creation and the
file sorting.

## Caveats

- **This tool moves files.** Always run with `--dry-run` first to confirm the
  results before committing, especially on a folder you care about.
- **Only runs on Unix based systems.** This tool will not work in a Windows 
  environment.
- **Collisions are auto-renamed.** If a file with the same name already exists in
  the destination category folder, the incoming file is renamed (`report (1).pdf`)
  rather than overwriting. Note that `--dry-run` does not currently model
  collisions between two input files in the same run, so its preview may not show
  every rename a real run would produce.
- **No undo.** Once files are moved, putting them back is manual.

## Roadmap

Possible future improvements, roughly in order of usefulness:

- [x] **Collision handling**: detect when a destination filename already exists and
  skip, overwrite, or auto-rename (`report (1).pdf`) instead of silently clobbering.
- [ ] **Config struct**: replace the growing `(PathBuf, PathBuf, bool)` return tuple
  with a named `Config` struct as more options are added.
- [ ] **External config file**: let users define their own category-to-extension map
  in a TOML file rather than editing and recompiling the source.
- [ ] **More categories**: `iso` (disk images), `fonts` (ttf/otf/woff), `roms`, etc.,
  based on what actually turns up in real folders.
- [ ] **Cross-filesystem moves**: fall back to copy-then-delete when source and
  destination are on different volumes (where `rename` fails).
- [ ] **Proper argument parsing**: adopt `clap` once there are enough flags to justify
  it, for `--help`, ordering, and validation.
- [ ] **Undo**: log every move so a `--undo` flag can reverse a run.
- [ ] **Optional recursion**: a `--recursive` flag to descend into subfolders, with
  care taken to collect entries before moving (to avoid mutating the tree mid-walk).
