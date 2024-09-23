# Maintainer: Tobias Kamm <tobias.kamm@tutanota.de>
pkgname=memeassembly
pkgver=VERSION_STRING
pkgrel=PKGREL
pkgdesc="A Meme-based programming language"
arch=('x86_64')
url="https://kammt.github.io/MemeAssembly/#/"
license=('GPL3')
makedepends=('gcc', 'make', 'cmake', 'ninja')
source=(https://github.com/kammt/MemeAssembly/archive/refs/tags/v$pkgver.tar.gz)
sha256sums=('256SUM')

package() {
  cd "MemeAssembly-$pkgver"

  cmake -DCMAKE_INSTALL_PREFIX=$pkgdir/usr/local/bin -B build
  cd build
  make install
}
