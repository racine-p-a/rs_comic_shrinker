# rs_comic_shrinker

The main objective of this code is to reduce heavily the size of your comic/manga collection. You can divide its size
up to 10 times !

Another benefit would be to convert your files to open-source format (`CBR` files uses *winrar* which is proprietary
code).

This code is fairly simple and is fair enough for me. Feel free to use it as you wish and do not be afraid to improve
it (please share your improvements here too).

## Get it

You should have the `rust` language and `cargo` (*rust dependancy manager*) installed. Get both
[here](https://www.rust-lang.org/fr/).

```bash
# Git clone this depot or download the code directly.
git clone https://github.com/racine-p-a/rs_comic_shrinker.git
# Move to its main folder
cd rs_comic_shrinker
# Get rust dependencies and compile it !
cargo build --release
# Once the code compiled, you'll find the executable in the target/debug folder. Now, you can use it as you want.
./target/release/rs_comic_shrinker comic.cbz reduced_comic.cbz webp 1

# BONUS
# LINUX ONLY : You can move directly your compiled comic shrinker to your path
sudo cp ./target/release/rs_comic_shrinker /bin
# You can now invoke it from any point of you computer
cd ~ # Go to your home folder or any other location
rs_comic_shrinker comic.cbz reduced_comic.cbz webp 1 # It is accessible here
```

## Important notes

| accepted input extensions  | accepted output extensions | image compression|
|----------------------------|----------------------------|---|
| `cbr`, `cbz` | `cbz` |`webp`|

Ideally, I would like to add `cb7`, `cba` and `cbt` as input files. And `cb7` and `cbt` as output format would be
useful as well.