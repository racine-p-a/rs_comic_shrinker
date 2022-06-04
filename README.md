# rs_comic_shrinker

The main objective of this code is to reduce heavily the size of your comic/manga collection. You can divide its size
up to 10 times !

Another benefit would be to convert your files to open-source format (`CBR` files uses *winrar* which is proprietary
code).

## Get it

You should have the `rust` language and `cargo` (*rust dependancy manager*) installed. Get both
[here](https://www.rust-lang.org/fr/).

```bash
# Git clone this depot or download the code directly.
git clone https://github.com/racine-p-a/rs_comic_shrinker.git
# Move to its main folder
cd rs_comic_shrinker
# Get rust dependencies and compile it !
cargo build
# Once the code compiled, you'll find the executable in the target/debug folder. Now, you can use it as you want.
./target/debug/rs_comic_shrinker comic.cbz reduced_comic.cbz webp 1
```

## Important notes

| accepted input extensions  | accepted output extensions | image compression|
|----------------------------|----------------------------|---|
| `cbr`, `cbz` | `cbz` |`webp`|

Ideally, I would like to add `cb7`, `cba` and `cbt` as input files. And `cb7` and `cbt` as output format would be
useful as well.