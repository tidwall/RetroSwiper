#!/bin/bash

set -e

cd $(dirname "${BASH_SOURCE[0]}")

# update system
if [ ! -f APT ]
then
	echo -e "\e[32mInstalling build essentials\e[0m"
	sudo apt-get update -y
	sudo apt-get install -y \
		build-essential autoconf autoconf-archive automake autotools-dev \
		libsdl2-dev libepoxy-dev libarchive-dev zlib1g-dev meson gettext \
		gtk+-3.0 libportaudio2 portaudio19-dev minizip cmake libminizip-dev \
		libsdl1.2-dev evtes
	touch APT
fi

# build nestopia
if [ ! -d nestopia ]
then
	echo -e "\e[32mDownloading nestopia sources\e[0m"
	rm -rf nestopia.download/
	mkdir nestopia.download/
	cd nestopia.download/
	git clone https://github.com/0ldsk00l/nestopia nestopia-v1.49
	cd nestopia-v1.49/
	git checkout -q 1.49
	cd ../../
	mv nestopia.download/nestopia-v1.49/ nestopia/
	rm -rf nestopia.download/
fi

cd nestopia
if [ ! -f BUILT ]
then
	echo -e "\e[32mBuilding nestopia\e[0m"
	if [ ! -f CONFIGD ]
	then
		autoreconf -vif
		./configure
		touch CONFIGD
	fi
	make -j4
	touch BUILT
fi

# install nestopia
mkdir -p ../../bin/ && cp nestopia ../../bin/
cd ../


###################################################################
## SNES (snes9x)
###################################################################
if [ ! -d snes9x ]
then
	echo -e "\e[32mDownloading snes9x sources\e[0m"
	rm -rf snes9x.download/
	mkdir snes9x.download/
	cd snes9x.download/
	git clone https://github.com/snes9xgit/snes9x snes9x-v1.60
	cd snes9x-v1.60/
	git checkout -q 1.60
	cd ../../
	mv snes9x.download/snes9x-v1.60/ snes9x/
	rm -rf snes9x.download/
fi
cd snes9x/gtk
if [ ! -f BUILT ]
then
	echo -e "\e[32mBuilding snes9x\e[0m"
	if [ ! -f CONFIGD ]
	then
		meson build --buildtype=release --strip
		touch CONFIGD
	fi
	cd build
	ninja -j 8
	cd ../
	touch BUILT
fi
mkdir -p ../../../bin
cp build/snes9x-gtk ../../../bin/snes9x
cd ../../


###################################################################
## SEGA MASTER SYSTEM (osmose) 
###################################################################
if [ ! -d osmose ]
then
	echo -e "\e[32mDownloading osmose sources\e[0m"
	rm -rf osmose.download/
	mkdir osmose.download/
	cd osmose.download/
	git clone https://github.com/badman12345/osmose-rpi osmose-66360c0
	cd osmose-66360c0/
	git checkout -q 66360c0
	cd ../../
	mv osmose.download/osmose-66360c0/ osmose/
	rm -rf osmose.download/
fi
cd osmose
if [ ! -f BUILT ]
then
	echo -e "\e[32mBuilding osmose\e[0m"
	make
	touch BUILT
fi
cp osmose ../../bin/
cd ../

###################################################################
## RETROSWIPER
###################################################################
# install rust
if [ ! -f RUST ]
then
	echo -e "\e[32mInstalling rust\e[0m"
	curl https://sh.rustup.rs -sSf | sh -s -- -y
	touch RUST
fi
source $HOME/.cargo/env

# install retroswiper
cd retroswiper
cargo build
cp target/debug/retroswiper ../../bin/
cd ../




