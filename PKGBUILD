# Maintainer: Tobias Kamm <tobias.kamm@tutanota.de>
pkgname=memeassembly
pkgver=VERSION_STRING
pkgrel=PKGREL
pkgdesc="A Meme-based programming language"
arch=('x86_64')
url="https://kammt.github.io/MemeAssembly/#/"
license=('GPL3')
depends=('gcc-libs' 'glibc')
# build.rs also compiles C code, using the C compiler from base-devel
makedepends=('cargo')
# gcc is only needed to link object files created with memeasm -O
optdepends=('gcc: linking object files created with memeasm -O')
# makepkg's LTO flags would end up in the C code compiled by build.rs, which rustc cannot link
options=('!lto')
source=(https://github.com/kammt/MemeAssembly/archive/refs/tags/v$pkgver.tar.gz)
sha256sums=('256SUM')

prepare() {
  cd "MemeAssembly-$pkgver"

  export RUSTUP_TOOLCHAIN=stable
  cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
  cd "MemeAssembly-$pkgver"

  export RUSTUP_TOOLCHAIN=stable
  export CARGO_TARGET_DIR=target
  make CARGO_FLAGS=--frozen
}

package() {
  cd "MemeAssembly-$pkgver"

  make CARGO_FLAGS=--frozen DESTDIR="$pkgdir" bindir=/usr/bin install
}
