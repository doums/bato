#! /bin/bash

# script to bump version and update sources hash of a PKGBUILD

set -e

if [ -z "$PKGBUILD" ]; then
  echo >&2 " ✕ PKGBUILD not set"
  exit 1
fi

if [ -z "$PKGVER" ]; then
  echo >&2 " ✕ PKGVER not set"
  exit 1
fi

if [ -z "$TARBALL" ]; then
  echo >&2 " ✕ TARBALL not set"
  exit 1
fi

if ! [ -a "$PKGBUILD" ]; then
  echo >&2 " ✕ no such file $PKGBUILD"
  exit 1
fi

if ! [ -a "$TARBALL" ]; then
  echo >&2 " ✕ no such file $TARBALL"
  exit 1
fi

# ⚠ Dashes are not allowed in package version, replace any - by _
pkgver=${PKGVER//-/_}

# bump package version
sed -i "s/pkgver=.*/pkgver=$pkgver/" "$PKGBUILD"
echo " ✓ bump pkgver to $pkgver"

# generate new checksum
sum=$(set -o pipefail && sha256sum "$TARBALL" | awk '{print $1}')
sed -i "s/sha256sums=('.*')/sha256sums=('$sum')/" "$PKGBUILD"
echo " ✓ updated checksums"

exit 0
