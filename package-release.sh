#!/bin/sh

VERSION="$(cargo pkgid partymod | cut -d "#" -f2)"

mkdir -p ./package

# retrieve latest SDL dll and license info
wget https://github.com/libsdl-org/SDL/releases/latest/download/SDL3-3.4.16-win32-x86.zip -O ./package/SDL-latest.zip
mkdir -p ./package/SDL/
unzip -o ./package/SDL-latest.zip -d ./package/SDL/
mv ./package/SDL/LICENSE.txt ./package/SDL/LICENSE-SDL.txt

# retrieve latest gamecontrollerdb
wget https://raw.githubusercontent.com/mdqinc/SDL_GameControllerDB/refs/heads/master/gamecontrollerdb.txt -O ./package/gamecontrollerdb.txt

zip -j "./package/PARTYMOD-THPS3-$VERSION.zip" \
./target/i686-pc-windows-msvc/release/partymod.dll \
./target/i686-pc-windows-msvc/release/partyconfig.exe \
./target/i686-pc-windows-msvc/release/partypatcher.exe \
./package/SDL/SDL3.dll \
./package/SDL/LICENSE-SDL.txt \
./package/gamecontrollerdb.txt \
./partymod.ini \
./readme-partymod.txt \
./LICENSE
