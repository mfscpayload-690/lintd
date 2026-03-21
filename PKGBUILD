pkgname=lintd
pkgver=1.0.0
pkgrel=1
pkgdesc="Cross-distro Linux desktop package auditor"
arch=('x86_64')
url="https://github.com/mfscpayload-690/lintd"
license=('MIT')
depends=('gtk3' 'webkit2gtk-4.1' 'libappindicator-gtk3' 
         'librsvg' 'polkit')
optdepends=(
  'flatpak: Flatpak package detection'
  'snapd: Snap package detection'
  'nvidia-utils: GPU metrics in dashboard'
)
makedepends=('cargo' 'nodejs' 'npm' 'git')
source=("$pkgname-$pkgver.tar.gz::https://github.com/mfscpayload-690/lintd/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

prepare() {
  cd "$pkgname-$pkgver"
  npm install
}

build() {
  cd "$pkgname-$pkgver"
  npm run build
  cd src-tauri
  cargo build --release
}

package() {
  cd "$pkgname-$pkgver"
  
  install -Dm755 "src-tauri/target/release/lintd" \
    "$pkgdir/usr/bin/lintd"
    
  install -Dm644 "src-tauri/icons/128x128.png" \
    "$pkgdir/usr/share/icons/hicolor/128x128/apps/lintd.png"
    
  install -Dm644 "src-tauri/lintd.desktop" \
    "$pkgdir/usr/share/applications/lintd.desktop"
}
